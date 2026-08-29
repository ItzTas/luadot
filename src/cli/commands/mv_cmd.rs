use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::crypt;
use crate::files::{self, Entry, LinkMode};
use crate::git;
use crate::lua::Shared;
use crate::output::{self, Tone};
use crate::utils::{self, Workspace};

#[derive(Debug, Args)]
pub struct MvArgs {
    #[arg(
        value_name = "PATH",
        required = true,
        num_args = 2..,
        help = "The managed paths to move, and where they go"
    )]
    pub paths: Vec<String>,
    #[arg(short = 'n', long, help = "Report what would happen, touching nothing")]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Moved {
    Relinked,
    Carried,
    Absent,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Counts {
    relinked: u32,
    carried: u32,
    absent: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct Move {
    entry: Entry,
    dest: PathBuf,
    was: PathBuf,
    now: PathBuf,
}

pub fn mv_cmd(args: MvArgs) -> Result<()> {
    let Workspace { config, home, repo } = utils::workspace("mv")?;

    let Some((dest, sources)) = args.paths.split_last().filter(|(_, rest)| !rest.is_empty()) else {
        bail!("mv: name the paths to move and where they go");
    };

    let moves = plan(&home, &repo, sources, dest)?;
    if moves.is_empty() {
        output::note("nothing to move");
        return Ok(());
    }
    check_wholes(&config, &repo, &moves)?;
    if args.dry_run {
        return foresee(&repo, &moves);
    }

    let mut counts = Counts::default();
    for one in &moves {
        counts.record(carry(&repo, one)?);
        output::entry(Tone::Good, "moved", shown(&repo, one));
    }

    let left: Vec<PathBuf> = moves
        .iter()
        .map(|one| one.entry.path().to_path_buf())
        .collect();
    let right: Vec<PathBuf> = moves.iter().map(|one| one.dest.clone()).collect();
    git::unstage("mv", &repo, &left)?;
    git::stage("mv", &repo, &right)?;
    output::note(counts.summary("moved", moves.len()));

    let automatic = {
        let config = utils::configured("mv", &config)?;
        utils::automatic(&config, &repo, &right)
    };
    git::auto("mv", &repo, automatic.commits, automatic.pushes)
}

fn foresee(repo: &Path, moves: &[Move]) -> Result<()> {
    let mut counts = Counts::default();
    for one in moves {
        counts.record(predict(one)?);
        output::entry(Tone::Muted, "move", shown(repo, one));
    }
    output::note(counts.summary("would move", moves.len()));

    Ok(())
}

fn shown(repo: &Path, one: &Move) -> String {
    format!(
        "{} -> {}",
        utils::relative(repo, one.entry.path()).display(),
        utils::relative(repo, &one.dest).display()
    )
}

fn check_wholes(shared: &Shared, repo: &Path, moves: &[Move]) -> Result<()> {
    let config = utils::configured("mv", shared)?;
    for one in moves {
        for path in [one.entry.path(), one.dest.as_path()] {
            let logical = crypt::logical(utils::relative(repo, path));
            if let Some(root) = config.unit_root(&logical) {
                bail!(
                    "mv: {} is inside {}, which is placed whole; adjust the rule before moving it",
                    utils::relative(repo, path).display(),
                    root.display()
                );
            }
        }
    }

    Ok(())
}

fn plan(home: &Path, repo: &Path, sources: &[String], dest: &str) -> Result<Vec<Move>> {
    let landing = landing(home, repo, dest)?;
    let into = landing.is_dir() && !files::is_template(&landing);
    if sources.len() > 1 && !into {
        bail!("mv: {dest} is not a directory of the repository");
    }

    let mut moves = Vec::new();
    for source in sources {
        let mirrored = utils::repo_path(home, repo, &absolute("mv", source)?)
            .with_context(|| format!("mv: cannot move {source}"))?;
        let root = utils::managed_path("mv", home, repo, source)?;
        let dest = match into {
            true => landing.join(named("mv", &root)?),
            false => renamed("mv", &mirrored, &root, &landing)?,
        };
        if dest != root && dest.starts_with(&root) {
            bail!(
                "mv: {} would land inside itself",
                utils::relative(repo, &root).display()
            );
        }
        moves.extend(expand(home, repo, &root, &dest)?);
    }
    check(repo, &moves)?;

    Ok(moves)
}

fn landing(home: &Path, repo: &Path, dest: &str) -> Result<PathBuf> {
    let target = absolute("mv", dest)?;

    utils::repo_path(home, repo, &target)
        .with_context(|| format!("mv: cannot move into {}", target.display()))
}

fn absolute(command: &str, path: &str) -> Result<PathBuf> {
    std::path::absolute(path).with_context(|| format!("{command}: invalid path {path}"))
}

fn renamed(command: &str, mirrored: &Path, root: &Path, landing: &Path) -> Result<PathBuf> {
    let stored = named(command, root)?;
    let plain = named(command, mirrored)?;
    let Some(suffix) = stored.strip_prefix(plain).filter(|rest| !rest.is_empty()) else {
        return Ok(landing.to_path_buf());
    };

    let name = named(command, landing)?;

    Ok(landing.with_file_name(format!("{name}{suffix}")))
}

fn named<'a>(command: &str, path: &'a Path) -> Result<&'a str> {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        bail!("{command}: {} has no name to move", path.display());
    };

    Ok(name)
}

fn expand(home: &Path, repo: &Path, root: &Path, dest: &Path) -> Result<Vec<Move>> {
    let mut moves = Vec::new();
    for entry in files::collect_entries("mv", root)? {
        let inside = entry.path().strip_prefix(root).unwrap_or(Path::new(""));
        let target = descend(dest, inside);
        let was = system(home, repo, &entry, entry.path())?;
        let now = system(home, repo, &entry, &target)?;
        moves.push(Move {
            entry,
            dest: target,
            was,
            now,
        });
    }

    Ok(moves)
}

fn system(home: &Path, repo: &Path, entry: &Entry, path: &Path) -> Result<PathBuf> {
    let plain = match entry {
        Entry::File(_) => path.to_path_buf(),
        Entry::Template(_) | Entry::Standalone(_) => {
            files::template_target(path).unwrap_or_else(|| path.to_path_buf())
        }
    };
    let logical = crypt::logical(utils::relative(repo, &plain));

    utils::system_path(home, repo, &repo.join(logical))
}

fn check(repo: &Path, moves: &[Move]) -> Result<()> {
    let mut seen: HashSet<&Path> = HashSet::new();
    for one in moves {
        let dest = utils::relative(repo, &one.dest).display();
        if one.dest.as_path() == one.entry.path() {
            bail!("mv: {dest} is already where it is");
        }
        if files::exists("mv", &one.dest)? {
            bail!("mv: {dest} already exists in the repository");
        }
        if files::exists("mv", &one.now)? {
            bail!("mv: {} already exists", one.now.display());
        }
        if !seen.insert(one.dest.as_path()) {
            bail!("mv: {dest} would be moved to more than once");
        }
    }

    Ok(())
}

fn carry(repo: &Path, one: &Move) -> Result<Moved> {
    files::create_parent("mv", &one.dest)?;
    std::fs::rename(one.entry.path(), &one.dest).with_context(|| {
        format!(
            "mv: failed to move {} to {}",
            one.entry.path().display(),
            one.dest.display()
        )
    })?;

    let moved = follow(one)?;
    repoint(one)?;
    files::prune_parents("mv", repo, one.entry.path())?;

    Ok(moved)
}

fn repoint(one: &Move) -> Result<()> {
    let Some(target) = points(&one.dest, &one.was)? else {
        return Ok(());
    };

    files::replace_file("mv", &one.dest, |staged| {
        files::link(LinkMode::Symbolic, &descend(&one.now, &target), staged)
    })
}

fn follow(one: &Move) -> Result<Moved> {
    if !files::exists("mv", &one.was)? {
        return Ok(Moved::Absent);
    }
    files::create_parent("mv", &one.now)?;

    let Some(inside) = points(&one.was, one.entry.path())? else {
        std::fs::rename(&one.was, &one.now).with_context(|| {
            format!(
                "mv: failed to move {} to {}",
                one.was.display(),
                one.now.display()
            )
        })?;
        return Ok(Moved::Carried);
    };

    files::replace_file("mv", &one.now, |staged| {
        files::link(LinkMode::Symbolic, &descend(&one.dest, &inside), staged)
    })?;
    std::fs::remove_file(&one.was)
        .with_context(|| format!("mv: failed to remove {}", one.was.display()))?;

    Ok(Moved::Relinked)
}

fn predict(one: &Move) -> Result<Moved> {
    if !files::exists("mv", &one.was)? {
        return Ok(Moved::Absent);
    }

    Ok(match points(&one.was, one.entry.path())?.is_some() {
        true => Moved::Relinked,
        false => Moved::Carried,
    })
}

fn points(link: &Path, root: &Path) -> Result<Option<PathBuf>> {
    let Some(target) = files::link_at("mv", link)? else {
        return Ok(None);
    };
    let Ok(inside) = target.strip_prefix(root) else {
        return Ok(None);
    };

    Ok(Some(inside.to_path_buf()))
}

fn descend(base: &Path, inside: &Path) -> PathBuf {
    if inside.as_os_str().is_empty() {
        return base.to_path_buf();
    }

    base.join(inside)
}

impl Counts {
    fn record(&mut self, moved: Moved) {
        match moved {
            Moved::Relinked => self.relinked += 1,
            Moved::Carried => self.carried += 1,
            Moved::Absent => self.absent += 1,
        }
    }

    fn summary(&self, verb: &str, total: usize) -> String {
        format!(
            "{verb} {total} file(s) ({} relinked, {} carried, {} not on the system)",
            self.relinked, self.carried, self.absent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arg(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    fn write(path: &Path, contents: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn a_rename_keeps_the_secret_form() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        write(&repo.join(".netrc.age"), "cipher");

        let moves = plan(
            &home,
            &repo,
            &[arg(&home.join(".netrc"))],
            &arg(&home.join(".config/netrc")),
        )
        .unwrap();

        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].dest, repo.join(".config/netrc.age"));
        assert_eq!(moves[0].now, home.join(".config/netrc"));
    }

    #[test]
    fn a_directory_moves_every_file_below() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        write(&repo.join(".config/nvim/init.lua"), "init");
        write(&repo.join(".config/nvim/lua/plugins.lua"), "plugins");

        let moves = plan(
            &home,
            &repo,
            &[arg(&home.join(".config/nvim"))],
            &arg(&home.join(".config/neovim")),
        )
        .unwrap();

        let dests: Vec<&Path> = moves.iter().map(|one| one.dest.as_path()).collect();
        assert_eq!(
            dests,
            [
                repo.join(".config/neovim/init.lua"),
                repo.join(".config/neovim/lua/plugins.lua"),
            ]
        );
    }

    #[test]
    fn several_paths_need_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        write(&repo.join(".vimrc"), "vim");
        write(&repo.join(".zshrc"), "zsh");

        let dest = arg(&home.join(".config/shell"));
        let err = plan(
            &home,
            &repo,
            &[arg(&home.join(".vimrc")), arg(&home.join(".zshrc"))],
            &dest,
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            err,
            format!("mv: {dest} is not a directory of the repository")
        );
    }

    #[test]
    fn an_existing_destination_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        write(&repo.join(".vimrc"), "vim");
        write(&repo.join(".config/vimrc"), "taken");

        let err = plan(
            &home,
            &repo,
            &[arg(&home.join(".vimrc"))],
            &arg(&home.join(".config/vimrc")),
        )
        .unwrap_err()
        .to_string();

        assert_eq!(err, "mv: .config/vimrc already exists in the repository");
    }

    #[test]
    fn a_system_symlink_follows_the_move() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        write(&repo.join(".vimrc"), "set number\n");
        std::fs::create_dir_all(&home).unwrap();
        std::os::unix::fs::symlink(repo.join(".vimrc"), home.join(".vimrc")).unwrap();

        let moves = plan(
            &home,
            &repo,
            &[arg(&home.join(".vimrc"))],
            &arg(&home.join(".config/vimrc")),
        )
        .unwrap();
        assert_eq!(carry(&repo, &moves[0]).unwrap(), Moved::Relinked);

        assert!(!home.join(".vimrc").exists());
        assert_eq!(
            std::fs::read_link(home.join(".config/vimrc")).unwrap(),
            repo.join(".config/vimrc")
        );
    }

    #[test]
    fn a_repository_symlink_follows_the_move() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        write(&home.join(".vimrc"), "set number\n");
        std::fs::create_dir_all(&repo).unwrap();
        std::os::unix::fs::symlink(home.join(".vimrc"), repo.join(".vimrc")).unwrap();

        let moves = plan(
            &home,
            &repo,
            &[arg(&home.join(".vimrc"))],
            &arg(&home.join(".config/vimrc")),
        )
        .unwrap();
        assert_eq!(carry(&repo, &moves[0]).unwrap(), Moved::Carried);

        assert_eq!(
            std::fs::read_link(repo.join(".config/vimrc")).unwrap(),
            home.join(".config/vimrc")
        );
        assert_eq!(
            std::fs::read_to_string(repo.join(".config/vimrc")).unwrap(),
            "set number\n"
        );
    }

    #[test]
    fn a_standalone_copy_travels_along() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        write(&repo.join(".vimrc"), "repository\n");
        write(&home.join(".vimrc"), "system\n");

        let moves = plan(
            &home,
            &repo,
            &[arg(&home.join(".vimrc"))],
            &arg(&home.join(".config/vimrc")),
        )
        .unwrap();
        assert_eq!(carry(&repo, &moves[0]).unwrap(), Moved::Carried);

        assert!(!home.join(".vimrc").exists());
        assert_eq!(
            std::fs::read_to_string(home.join(".config/vimrc")).unwrap(),
            "system\n"
        );
    }

    #[test]
    fn a_template_moves_with_its_output() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let template = repo.join(".zshrc.luadot");
        write(&template.join("luadot.lua"), r#"return "generated\n""#);
        write(&home.join(".zshrc"), "generated\n");

        let moves = plan(
            &home,
            &repo,
            &[arg(&home.join(".zshrc"))],
            &arg(&home.join(".zprofile")),
        )
        .unwrap();

        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].dest, repo.join(".zprofile.luadot"));
        assert_eq!(carry(&repo, &moves[0]).unwrap(), Moved::Carried);
        assert!(repo.join(".zprofile.luadot/luadot.lua").exists());
        assert_eq!(
            std::fs::read_to_string(home.join(".zprofile")).unwrap(),
            "generated\n"
        );
    }
}
