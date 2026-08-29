use std::collections::HashSet;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::backup::Backup;
use crate::crypt;
use crate::files::{self, ConflictPolicy, LinkMode, Placement, SyncOutcome};
use crate::git;
use crate::lua::Config;
use crate::output::{self, Tone};
use crate::utils::{self, Workspace};

use super::super::constants::{ADD_COMMAND, TAKE_COMMAND};

#[derive(Debug, Args)]
pub struct AddArgs {
    #[arg(
        value_name = "PATH",
        help = "The files or directories to start managing; with none, the files an `auto` rule covers"
    )]
    pub paths: Vec<String>,
}

#[derive(Debug, Args)]
pub struct TakeArgs {
    #[arg(
        value_name = "PATH",
        help = "The managed files or directories to store as the system holds them; with none, everything the repository holds"
    )]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Add,
    Replace,
}

type Pair = (PathBuf, PathBuf);

#[derive(Debug, PartialEq, Eq)]
struct Whole {
    source: PathBuf,
    dest: PathBuf,
}

impl Mode {
    fn command(self) -> &'static str {
        match self {
            Self::Add => ADD_COMMAND,
            Self::Replace => TAKE_COMMAND,
        }
    }
}

pub fn add_cmd(args: AddArgs) -> Result<()> {
    run(Mode::Add, &args.paths)
}

pub fn take_cmd(args: TakeArgs) -> Result<()> {
    run(Mode::Replace, &args.paths)
}

fn run(mode: Mode, paths: &[String]) -> Result<()> {
    let command = mode.command();
    let Workspace { config, home, repo } = utils::workspace(command)?;
    let config = utils::configured(command, &config)?;

    let (pairs, wholes) = plan(mode, &home, &repo, paths, &config)?;
    let lock = config.crypt_lock();
    require_plugins(mode, &config, lock, &repo, &pairs)?;
    let mut backup = opened(mode, paths, &config)?;

    let mut stored = Vec::with_capacity(pairs.len() + wholes.len());
    let mut replaced = 0;
    for (source, dest) in pairs {
        save(command, &mut backup, &dest)?;
        let outcome = link_into_repo(mode, &config, lock, &repo, &source, &dest)?;
        if outcome == SyncOutcome::Replaced {
            replaced += 1;
        }
        announce(outcome, utils::relative(&repo, &dest).display());
        stored.push(dest);
    }
    for whole in wholes {
        save(command, &mut backup, &whole.dest)?;
        let outcome = store_whole(mode, &config, &repo, &whole)?;
        if outcome == SyncOutcome::Replaced {
            replaced += 1;
        }
        announce(outcome, utils::relative(&repo, &whole.dest).display());
        stored.push(whole.dest);
    }

    track_in_lfs(mode, &config, &repo)?;
    git::stage(command, &repo, &stored)?;
    report(mode, &stored, replaced, paths);
    if let Some(backup) = backup.as_ref() {
        backup.finish()?;
    }

    let automatic = utils::automatic(&config, &repo, &stored);
    git::auto(command, &repo, automatic.commits, automatic.pushes)
}

fn opened(mode: Mode, sources: &[String], config: &Config) -> Result<Option<Backup>> {
    if mode != Mode::Replace || !sources.is_empty() || !config.backup() {
        return Ok(None);
    }

    Backup::open(TAKE_COMMAND, config.backup_dir(), config.retention()).map(Some)
}

fn save(command: &str, backup: &mut Option<Backup>, dest: &Path) -> Result<()> {
    let Some(backup) = backup.as_mut() else {
        return Ok(());
    };
    if !files::exists(command, dest)? {
        return Ok(());
    }

    backup.save(dest)
}

fn announce(outcome: SyncOutcome, path: impl Display) {
    if outcome != SyncOutcome::Created {
        return output::report(outcome, path);
    }

    output::entry(Tone::Good, "added", path);
}

fn report(mode: Mode, stored: &[PathBuf], replaced: usize, sources: &[String]) {
    if stored.is_empty() {
        output::note(nothing(mode, sources));
        return;
    }

    let total = stored.len();
    if mode == Mode::Add {
        output::note(format!("added {total} file(s)"));
        return;
    }

    output::note(format!(
        "took {total} file(s) ({} added, {replaced} replaced)",
        total - replaced
    ));
}

fn nothing(mode: Mode, sources: &[String]) -> String {
    if !sources.is_empty() {
        return format!(
            "nothing to {}, the rules leave every path out",
            mode.command()
        );
    }
    if mode == Mode::Add {
        return "nothing to add, the `auto` rules cover nothing new".to_string();
    }

    "nothing to take, the repository holds nothing the system has".to_string()
}

fn track_in_lfs(mode: Mode, config: &Config, repo: &Path) -> Result<()> {
    let command = mode.command();
    let patterns = config.lfs_patterns();
    if !patterns.is_empty() {
        if !git::lfs_available() {
            output::warn("git-lfs is not on your PATH, the files it tracks are stored as they are");
        }
        git::install_lfs(command, repo, config.lfs())?;
    }
    if !git::sync_attributes(command, repo, &patterns)? {
        return Ok(());
    }
    git::refresh_info(command, repo)?;

    let path = git::attributes_path(command, repo)?;
    if path.exists() {
        return git::stage(command, repo, &[path]);
    }

    git::unstage(command, repo, &[path])
}

fn require_plugins(
    mode: Mode,
    config: &Config,
    lock: crypt::Lock,
    repo: &Path,
    pairs: &[Pair],
) -> Result<()> {
    let encrypts = pairs.iter().any(|(_, dest)| {
        matches!(
            crypt::split(utils::relative(repo, dest)),
            Some((stripped, crypt::Backend::Age)) if config.encrypt(&stripped)
        )
    });
    if !encrypts {
        return Ok(());
    }

    crypt::require_recipient_plugins(
        mode.command(),
        crypt::Backend::Age,
        lock,
        config.crypt_secrets().recipients(),
    )
}

fn plan(
    mode: Mode,
    home: &Path,
    repo: &Path,
    sources: &[String],
    config: &Config,
) -> Result<(Vec<Pair>, Vec<Whole>)> {
    let command = mode.command();
    let mut excludes = git::Excludes::open(command, repo)?;
    let mut wholes: Vec<Whole> = Vec::new();

    let found = match sources.is_empty() {
        true => sweep(mode, home, repo, config, &mut excludes)?,
        false => named(command, sources)?,
    };
    let pairs = gather(mode, home, repo, found, config, &mut excludes, &mut wholes)?;
    check_conflicts(mode, &pairs, &wholes)?;

    Ok((pairs, wholes))
}

fn named(command: &str, sources: &[String]) -> Result<Vec<PathBuf>> {
    sources
        .iter()
        .map(|source| {
            std::path::absolute(source).with_context(|| format!("{command}: invalid path {source}"))
        })
        .collect()
}

fn sweep(
    mode: Mode,
    home: &Path,
    repo: &Path,
    config: &Config,
    excludes: &mut git::Excludes,
) -> Result<Vec<PathBuf>> {
    if mode == Mode::Add {
        return adopted(mode.command(), home, repo, config, excludes);
    }

    held(mode.command(), home, repo, config)
}

fn gather(
    mode: Mode,
    home: &Path,
    repo: &Path,
    sources: Vec<PathBuf>,
    config: &Config,
    excludes: &mut git::Excludes,
    wholes: &mut Vec<Whole>,
) -> Result<Vec<Pair>> {
    let mut pairs: Vec<Pair> = Vec::new();
    for source in sources {
        pairs.extend(collect(mode, home, repo, source, config, excludes, wholes)?);
    }

    Ok(pairs)
}

fn held(command: &str, home: &Path, repo: &Path, config: &Config) -> Result<Vec<PathBuf>> {
    let files = utils::managed_files(command, repo, repo, |relative| {
        config.is_ignored(&crypt::logical(relative))
    })?;

    let mut sources = Vec::new();
    for one in utils::units(command, config, repo, files)? {
        let relative = match &one {
            utils::Managed::Unit(unit) => utils::relative(repo, unit.root()).to_path_buf(),
            utils::Managed::File(file) => crypt::logical(utils::relative(repo, file)),
        };

        let source = home.join(relative);
        if files::exists(command, &source)? {
            sources.push(source);
        }
    }

    Ok(sources)
}

fn adopted(
    command: &str,
    home: &Path,
    repo: &Path,
    config: &Config,
    excludes: &mut git::Excludes,
) -> Result<Vec<PathBuf>> {
    let mut sources: Vec<PathBuf> = Vec::new();
    for file in utils::adoptable(command, home, repo, config, excludes)? {
        let relative = utils::managed_relative(home, &file)?;
        let source = match config.unit_root(&relative) {
            Some(root) => home.join(root),
            None => file,
        };

        if sources.contains(&source) || utils::repo_path(home, repo, &source)?.exists() {
            continue;
        }
        sources.push(source);
    }

    Ok(sources)
}

fn collect(
    mode: Mode,
    home: &Path,
    repo: &Path,
    source: PathBuf,
    config: &Config,
    excludes: &mut git::Excludes,
    wholes: &mut Vec<Whole>,
) -> Result<Vec<Pair>> {
    let command = mode.command();
    let relative = utils::managed_relative(home, &source)
        .with_context(|| format!("{command}: cannot manage {}", source.display()))?;

    if source.is_dir() {
        check_gitignore(mode, home, &source, git::Kind::Directory, excludes)?;
        if let Some(root) = config.unit_root(&relative) {
            check_inside(mode, home, &source, &relative, &root)?;
            if let Some(whole) = whole_dir(mode, home, repo, &source, true, config, excludes)? {
                wholes.push(whole);
            }
            return Ok(Vec::new());
        }
        return collect_dir(mode, home, repo, &source, config, excludes, wholes);
    }
    if source.is_file() {
        if let Some(root) = config.unit_root(&crypt::logical(&relative)) {
            check_inside(mode, home, &source, &relative, &root)?;
        }
        check_gitignore(mode, home, &source, git::Kind::File, excludes)?;
        check_template(mode, home, repo, &source)?;
        let Some(pair) = pair(home, repo, source, config, excludes)? else {
            return Ok(Vec::new());
        };
        check_managed(mode, &pair.1)?;
        return Ok(vec![pair]);
    }
    bail!("{command}: {} is not a file or directory", source.display())
}

fn check_inside(
    mode: Mode,
    home: &Path,
    source: &Path,
    relative: &Path,
    root: &Path,
) -> Result<()> {
    if root == relative {
        return Ok(());
    }

    bail!(
        "{}: {} sits inside {}, which is placed whole; run `luadot {} {}`",
        mode.command(),
        source.display(),
        root.display(),
        mode.command(),
        home.join(root).display()
    )
}

fn collect_dir(
    mode: Mode,
    home: &Path,
    repo: &Path,
    dir: &Path,
    config: &Config,
    excludes: &mut git::Excludes,
    wholes: &mut Vec<Whole>,
) -> Result<Vec<Pair>> {
    utils::repo_path(home, repo, dir)?;

    let mut files = Vec::new();
    walk(mode, dir, &mut files)?;
    files.sort();

    let mut roots: Vec<PathBuf> = Vec::new();
    let mut pairs = Vec::new();
    for file in files {
        let relative = utils::managed_relative(home, &file)?;
        if let Some(root) = config.unit_root(&relative).filter(|root| *root != relative) {
            let root_dir = home.join(&root);
            if !roots.contains(&root_dir) {
                if let Some(whole) =
                    whole_dir(mode, home, repo, &root_dir, false, config, excludes)?
                {
                    wholes.push(whole);
                }
                roots.push(root_dir);
            }
            continue;
        }
        if let Some(template) = template_for(home, repo, &file)? {
            output::warn(format!(
                "{} is produced by {}, leaving it out",
                file.display(),
                template.display()
            ));
            continue;
        }
        if let Some(pair) = pair(home, repo, file, config, excludes)? {
            pairs.push(pair);
        }
    }

    if mode == Mode::Add {
        return Ok(pairs);
    }

    Ok(pairs
        .into_iter()
        .filter(|(_, dest)| dest.exists())
        .collect())
}

fn whole_dir(
    mode: Mode,
    home: &Path,
    repo: &Path,
    source: &Path,
    direct: bool,
    config: &Config,
    excludes: &mut git::Excludes,
) -> Result<Option<Whole>> {
    let command = mode.command();
    let relative = utils::managed_relative(home, source)?;
    let dest = utils::repo_path(home, repo, source)?;
    utils::whole_link(command, config, &relative)?;

    if mode == Mode::Replace && !dest.exists() {
        if direct {
            check_managed(mode, &dest)?;
        }
        return Ok(None);
    }

    let mut files = Vec::new();
    walk(mode, source, &mut files)?;
    files.sort();
    for file in files {
        check_whole_member(mode, home, repo, &relative, &file, config, excludes)?;
    }

    Ok(Some(Whole {
        source: source.to_path_buf(),
        dest,
    }))
}

fn check_whole_member(
    mode: Mode,
    home: &Path,
    repo: &Path,
    root: &Path,
    file: &Path,
    config: &Config,
    excludes: &mut git::Excludes,
) -> Result<()> {
    let command = mode.command();
    if let Some(template) = template_for(home, repo, file)? {
        bail!(
            "{command}: {} is placed whole, but {} is produced by {}",
            root.display(),
            file.display(),
            template.display()
        );
    }

    let dest = utils::repo_path(home, repo, file)?;
    let relative = utils::relative(repo, &dest);
    if config.is_ignored(relative) || excludes.excluded(relative, git::Kind::File)? {
        bail!(
            "{command}: {} is placed whole, but the rules leave {} out",
            root.display(),
            relative.display()
        );
    }
    if config.encrypt(relative) {
        bail!(
            "{command}: {} is placed whole and cannot hold the encrypted {}",
            root.display(),
            relative.display()
        );
    }

    Ok(())
}

fn check_managed(mode: Mode, dest: &Path) -> Result<()> {
    if mode == Mode::Add || dest.exists() {
        return Ok(());
    }

    bail!(
        "{TAKE_COMMAND}: {} is not in the repository; run `luadot add` to start managing it",
        dest.display()
    )
}

fn check_template(mode: Mode, home: &Path, repo: &Path, source: &Path) -> Result<()> {
    let Some(template) = template_for(home, repo, source)? else {
        return Ok(());
    };

    bail!(
        "{}: {} is produced by {}; run `luadot edit` on it instead",
        mode.command(),
        source.display(),
        template.display()
    )
}

fn template_for(home: &Path, repo: &Path, source: &Path) -> Result<Option<PathBuf>> {
    let dest = utils::repo_path(home, repo, source)?;

    Ok(files::template_dir(&dest).filter(|path| std::fs::symlink_metadata(path).is_ok()))
}

fn check_gitignore(
    mode: Mode,
    home: &Path,
    source: &Path,
    kind: git::Kind,
    excludes: &mut git::Excludes,
) -> Result<()> {
    let relative = utils::managed_relative(home, source)?;
    if !excludes.excluded(&relative, kind)? {
        return Ok(());
    }

    bail!(
        "{}: {} lands on {}, which the repository's ignore rules exclude",
        mode.command(),
        source.display(),
        relative.display()
    )
}

fn pair(
    home: &Path,
    repo: &Path,
    source: PathBuf,
    config: &Config,
    excludes: &mut git::Excludes,
) -> Result<Option<(PathBuf, PathBuf)>> {
    let dest = utils::repo_path(home, repo, &source)?;

    let relative = utils::relative(repo, &dest);
    if config.is_ignored(relative) || excludes.excluded(relative, git::Kind::File)? {
        return Ok(None);
    }
    if !config.encrypt(relative) {
        return Ok(Some((source, dest)));
    }

    let stored = crypt::stored(&dest, config.crypt_backend());
    Ok(Some((source, stored)))
}

fn walk(mode: Mode, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let command = mode.command();
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("{command}: failed to read {}", dir.display()))?;
    for entry in entries {
        let entry =
            entry.with_context(|| format!("{command}: failed to read {}", dir.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("{command}: failed to inspect {}", entry.path().display()))?;
        let path = entry.path();
        if file_type.is_dir() {
            walk(mode, &path, out)?;
            continue;
        }
        if file_type.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

fn check_conflicts(mode: Mode, pairs: &[Pair], wholes: &[Whole]) -> Result<()> {
    let mut seen: HashSet<&Path> = HashSet::new();
    let dests = pairs
        .iter()
        .map(|(_, dest)| dest.as_path())
        .chain(wholes.iter().map(|whole| whole.dest.as_path()));
    for dest in dests {
        if mode == Mode::Add && dest.exists() {
            bail!(
                "{ADD_COMMAND}: {} already exists in the repository; run `luadot take` to store what the system holds",
                dest.display()
            );
        }
        if !seen.insert(dest) {
            bail!(
                "{}: {} would be added more than once",
                mode.command(),
                dest.display()
            );
        }
    }
    Ok(())
}

fn link_into_repo(
    mode: Mode,
    config: &Config,
    lock: crypt::Lock,
    repo: &Path,
    source: &Path,
    dest: &Path,
) -> Result<SyncOutcome> {
    let command = mode.command();
    let outcome = match files::exists(command, dest)? {
        true => SyncOutcome::Replaced,
        false => SyncOutcome::Created,
    };

    let relative = utils::relative(repo, dest);
    if let Some((stripped, backend)) = crypt::split(relative)
        && config.encrypt(&stripped)
    {
        let contents = files::read_contents(command, source)?;
        files::replace_file(command, dest, |staged| {
            crypt::encrypt_contents(
                command,
                backend,
                lock,
                config.crypt_secrets().recipients(),
                &contents,
                staged,
            )
        })?;

        return Ok(outcome);
    }

    let placement = config.placement(relative);
    if placement.link() == LinkMode::Symbolic {
        store_and_point_at(command, placement, source, dest)?;
        return Ok(outcome);
    }

    files::replace_file(command, dest, |staged| {
        files::link(placement.link(), source, staged)
    })?;

    Ok(outcome)
}

fn store_and_point_at(
    command: &str,
    placement: Placement,
    system: &Path,
    stored: &Path,
) -> Result<()> {
    files::replace_file(command, stored, |staged| {
        files::link(LinkMode::Copy, system, staged)
    })?;

    files::sync_file(ConflictPolicy::Overwrite, placement, stored, system).map(|_| ())
}

fn store_whole(mode: Mode, config: &Config, repo: &Path, whole: &Whole) -> Result<SyncOutcome> {
    let command = mode.command();
    let relative = utils::relative(repo, &whole.dest);
    let link = utils::whole_link(command, config, relative)?;

    if files::link_at(command, &whole.source)?.as_deref() == Some(whole.dest.as_path()) {
        return Ok(SyncOutcome::AlreadySynced);
    }

    let outcome = match files::exists(command, &whole.dest)? {
        true => SyncOutcome::Replaced,
        false => SyncOutcome::Created,
    };
    if outcome == SyncOutcome::Replaced {
        files::remove_entry(command, &whole.dest)?;
    }
    files::copy_tree(command, &whole.source, &whole.dest)?;

    if link == LinkMode::Symbolic {
        swap_for_link(command, &whole.source, &whole.dest)?;
    }

    Ok(outcome)
}

fn swap_for_link(command: &str, system: &Path, stored: &Path) -> Result<()> {
    let mut name = system.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{command}-{}", std::process::id()));
    let aside = system.with_file_name(name);

    std::fs::rename(system, &aside)
        .with_context(|| format!("{command}: failed to move {} aside", system.display()))?;
    if let Err(err) = std::os::unix::fs::symlink(stored, system) {
        let _ = std::fs::rename(&aside, system);
        return Err(err).with_context(|| {
            format!(
                "{command}: failed to symlink {} -> {}",
                system.display(),
                stored.display()
            )
        });
    }

    std::fs::remove_dir_all(&aside)
        .with_context(|| format!("{command}: failed to remove {}", aside.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua;

    fn arg(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn plan_drops_ignored_files() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        let kept = home.join(".vimrc");
        let ignored = home.join(".vimrc.swp");
        std::fs::write(&kept, "a").unwrap();
        std::fs::write(&ignored, "b").unwrap();

        let config = lua::from_source(r#"ld.rules({ match = "*.swp", track = "never" })"#).unwrap();
        let (pairs, _) = plan(
            Mode::Add,
            &home,
            &repo,
            &[arg(&kept), arg(&ignored)],
            &config,
        )
        .unwrap();

        assert_eq!(pairs, vec![(kept, repo.join(".vimrc"))]);
    }

    #[test]
    fn plan_refuses_a_file_outside_home() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let source = dir.path().join("etc/pacman.conf");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "x").unwrap();

        let err = format!(
            "{:#}",
            plan(Mode::Add, &home, &repo, &[arg(&source)], &Config::default()).unwrap_err()
        );

        assert_eq!(
            err,
            format!(
                "add: cannot manage {}: outside your home directory {}",
                source.display(),
                home.display()
            )
        );
    }

    #[test]
    fn plan_stores_the_backend_extension() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        let source = home.join(".netrc");
        std::fs::write(&source, "machine example").unwrap();

        let config = lua::from_source(
            r#"
            ld.crypt.backend("gpg")
            ld.rules({ match = ".netrc", encrypt = true })
            "#,
        )
        .unwrap();
        let (pairs, _) = plan(Mode::Add, &home, &repo, &[arg(&source)], &config).unwrap();

        assert_eq!(pairs, vec![(source, repo.join(".netrc.gpg"))]);
    }

    #[test]
    fn plan_walks_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = home.join(".config").join("nvim");
        std::fs::create_dir_all(nvim.join("lua")).unwrap();
        let init = nvim.join("init.lua");
        let plugins = nvim.join("lua").join("plugins.lua");
        std::fs::write(&init, "init").unwrap();
        std::fs::write(&plugins, "plugins").unwrap();

        let (pairs, _) = plan(Mode::Add, &home, &repo, &[arg(&nvim)], &Config::default()).unwrap();

        assert_eq!(
            pairs,
            vec![
                (init, repo.join(".config/nvim/init.lua")),
                (plugins, repo.join(".config/nvim/lua/plugins.lua")),
            ]
        );
    }

    #[test]
    fn plan_keeps_a_whole_directory_as_one_unit() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = home.join(".config/nvim");
        std::fs::create_dir_all(nvim.join("lua")).unwrap();
        std::fs::write(nvim.join("init.lua"), "init").unwrap();
        std::fs::write(nvim.join("lua/plugins.lua"), "plugins").unwrap();

        let config = lua::from_source(
            r#"ld.rules({ match = ".config/nvim", whole = true, link = "symbolic" })"#,
        )
        .unwrap();
        let (pairs, wholes) = plan(Mode::Add, &home, &repo, &[arg(&nvim)], &config).unwrap();

        assert_eq!(pairs, vec![]);
        assert_eq!(
            wholes,
            vec![Whole {
                source: nvim,
                dest: repo.join(".config/nvim"),
            }]
        );
    }

    #[test]
    fn plan_finds_a_whole_directory_under_a_walked_parent() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = home.join(".config/nvim");
        std::fs::create_dir_all(&nvim).unwrap();
        std::fs::write(nvim.join("init.lua"), "init").unwrap();
        std::fs::write(home.join(".config/starship.toml"), "prompt").unwrap();

        let config = lua::from_source(
            r#"ld.rules({ match = ".config/nvim", whole = true, link = "symbolic" })"#,
        )
        .unwrap();
        let (pairs, wholes) = plan(
            Mode::Add,
            &home,
            &repo,
            &[arg(&home.join(".config"))],
            &config,
        )
        .unwrap();

        assert_eq!(
            pairs,
            vec![(
                home.join(".config/starship.toml"),
                repo.join(".config/starship.toml")
            )]
        );
        assert_eq!(
            wholes,
            vec![Whole {
                source: nvim,
                dest: repo.join(".config/nvim"),
            }]
        );
    }

    #[test]
    fn plan_refuses_a_path_inside_a_whole_directory() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = home.join(".config/nvim");
        std::fs::create_dir_all(&nvim).unwrap();
        let init = nvim.join("init.lua");
        std::fs::write(&init, "init").unwrap();

        let config = lua::from_source(
            r#"ld.rules({ match = ".config/nvim", whole = true, link = "symbolic" })"#,
        )
        .unwrap();
        let err = format!(
            "{:#}",
            plan(Mode::Add, &home, &repo, &[arg(&init)], &config).unwrap_err()
        );

        assert_eq!(
            err,
            format!(
                "add: {} sits inside .config/nvim, which is placed whole; run `luadot add {}`",
                init.display(),
                nvim.display()
            )
        );
    }

    #[test]
    fn plan_refuses_a_whole_directory_leaving_a_file_out() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = home.join(".config/nvim");
        std::fs::create_dir_all(&nvim).unwrap();
        std::fs::write(nvim.join("init.lua"), "init").unwrap();
        std::fs::write(nvim.join("init.lua.swp"), "swap").unwrap();

        let config = lua::from_source(
            r#"
            ld.rules({
              { match = ".config/nvim", whole = true, link = "symbolic" },
              { match = "**/*.swp", track = "never" },
            })
            "#,
        )
        .unwrap();
        let err = format!(
            "{:#}",
            plan(Mode::Add, &home, &repo, &[arg(&nvim)], &config).unwrap_err()
        );

        assert_eq!(
            err,
            "add: .config/nvim is placed whole, but the rules leave .config/nvim/init.lua.swp out"
        );
    }

    #[test]
    fn store_whole_swaps_the_directory_for_a_link() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = home.join(".config/nvim");
        std::fs::create_dir_all(nvim.join("lua")).unwrap();
        std::fs::write(nvim.join("init.lua"), "init").unwrap();
        std::fs::write(nvim.join("lua/plugins.lua"), "plugins").unwrap();

        let config = lua::from_source(
            r#"ld.rules({ match = ".config/nvim", whole = true, link = "symbolic" })"#,
        )
        .unwrap();
        let whole = Whole {
            source: nvim.clone(),
            dest: repo.join(".config/nvim"),
        };

        let outcome = store_whole(Mode::Add, &config, &repo, &whole).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(
            std::fs::read_link(&nvim).unwrap(),
            repo.join(".config/nvim")
        );
        assert_eq!(
            std::fs::read_to_string(repo.join(".config/nvim/lua/plugins.lua")).unwrap(),
            "plugins"
        );

        let again = store_whole(Mode::Replace, &config, &repo, &whole).unwrap();
        assert_eq!(again, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn store_whole_copies_the_directory_without_touching_home() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let gnupg = home.join(".gnupg");
        std::fs::create_dir_all(&gnupg).unwrap();
        std::fs::write(gnupg.join("gpg.conf"), "keyserver").unwrap();

        let config =
            lua::from_source(r#"ld.rules({ match = ".gnupg", whole = true, link = "copy" })"#)
                .unwrap();
        let whole = Whole {
            source: gnupg.clone(),
            dest: repo.join(".gnupg"),
        };

        let outcome = store_whole(Mode::Add, &config, &repo, &whole).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert!(!std::fs::symlink_metadata(&gnupg).unwrap().is_symlink());
        assert_eq!(
            std::fs::read_to_string(repo.join(".gnupg/gpg.conf")).unwrap(),
            "keyserver"
        );
    }

    #[test]
    fn add_points_at_take_when_held() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&repo).unwrap();
        let source = home.join(".bashrc");
        let dest = repo.join(".bashrc");
        std::fs::write(&source, "system").unwrap();
        std::fs::write(&dest, "repository").unwrap();

        let err = format!(
            "{:#}",
            plan(Mode::Add, &home, &repo, &[arg(&source)], &Config::default()).unwrap_err()
        );

        assert_eq!(
            err,
            format!(
                "add: {} already exists in the repository; run `luadot take` to store what the system holds",
                dest.display()
            )
        );
    }

    #[test]
    fn take_reaches_a_held_file() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&repo).unwrap();
        let source = home.join(".bashrc");
        let dest = repo.join(".bashrc");
        std::fs::write(&source, "system").unwrap();
        std::fs::write(&dest, "repository").unwrap();

        let (pairs, _) = plan(
            Mode::Replace,
            &home,
            &repo,
            &[arg(&source)],
            &Config::default(),
        )
        .unwrap();

        assert_eq!(pairs, vec![(source, dest)]);
    }

    #[test]
    fn take_refuses_an_unheld_file() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        let source = home.join(".bashrc");
        std::fs::write(&source, "system").unwrap();

        let err = format!(
            "{:#}",
            plan(
                Mode::Replace,
                &home,
                &repo,
                &[arg(&source)],
                &Config::default()
            )
            .unwrap_err()
        );

        assert_eq!(
            err,
            format!(
                "take: {} is not in the repository; run `luadot add` to start managing it",
                repo.join(".bashrc").display()
            )
        );
    }

    #[test]
    fn take_skips_an_unheld_directory() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = home.join(".config").join("nvim");
        std::fs::create_dir_all(&nvim).unwrap();
        std::fs::create_dir_all(repo.join(".config/nvim")).unwrap();
        let managed = nvim.join("init.lua");
        let fresh = nvim.join("scratch.lua");
        std::fs::write(&managed, "init").unwrap();
        std::fs::write(&fresh, "scratch").unwrap();
        std::fs::write(repo.join(".config/nvim/init.lua"), "stored").unwrap();

        let (pairs, _) = plan(
            Mode::Replace,
            &home,
            &repo,
            &[arg(&nvim)],
            &Config::default(),
        )
        .unwrap();

        assert_eq!(pairs, vec![(managed, repo.join(".config/nvim/init.lua"))]);
    }

    #[test]
    fn take_with_no_path_reaches_what_the_repository_holds() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(repo.join(".config/nvim")).unwrap();
        std::fs::write(home.join(".bashrc"), "system").unwrap();
        std::fs::write(home.join(".netrc"), "machine example").unwrap();
        std::fs::write(repo.join(".bashrc"), "repository").unwrap();
        std::fs::write(repo.join(".netrc.age"), "cipher").unwrap();
        std::fs::write(repo.join(".config/nvim/init.lua"), "stored").unwrap();

        let config = lua::from_source(r#"ld.rules({ match = ".netrc", encrypt = true })"#).unwrap();
        let (pairs, wholes) = plan(Mode::Replace, &home, &repo, &[], &config).unwrap();

        assert_eq!(
            pairs,
            vec![
                (home.join(".bashrc"), repo.join(".bashrc")),
                (home.join(".netrc"), repo.join(".netrc.age")),
            ]
        );
        assert_eq!(wholes, vec![]);
    }

    #[test]
    fn take_with_no_path_reaches_a_whole_directory() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = home.join(".config/nvim");
        std::fs::create_dir_all(&nvim).unwrap();
        std::fs::create_dir_all(repo.join(".config/nvim")).unwrap();
        std::fs::write(nvim.join("init.lua"), "system").unwrap();
        std::fs::write(repo.join(".config/nvim/init.lua"), "stored").unwrap();

        let config = lua::from_source(
            r#"ld.rules({ match = ".config/nvim", whole = true, link = "symbolic" })"#,
        )
        .unwrap();
        let (pairs, wholes) = plan(Mode::Replace, &home, &repo, &[], &config).unwrap();

        assert_eq!(pairs, vec![]);
        assert_eq!(
            wholes,
            vec![Whole {
                source: nvim,
                dest: repo.join(".config/nvim"),
            }]
        );
    }

    #[test]
    fn linking_replaces_what_was_held() {
        use std::os::unix::fs::MetadataExt;

        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        let source = dir.path().join(".bashrc");
        let dest = repo.join(".bashrc");
        std::fs::write(&source, "system").unwrap();
        std::fs::write(&dest, "stale").unwrap();

        let outcome = link_into_repo(
            Mode::Replace,
            &Config::default(),
            crypt::Lock::default(),
            &repo,
            &source,
            &dest,
        )
        .unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "system");

        let (a, b) = (
            std::fs::metadata(&source).unwrap(),
            std::fs::metadata(&dest).unwrap(),
        );
        assert_eq!((a.dev(), a.ino()), (b.dev(), b.ino()));
    }
}
