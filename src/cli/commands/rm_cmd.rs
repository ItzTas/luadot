use std::fs::Metadata;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::backup::Backup;
use crate::crypt;
use crate::files::{self, Entry};
use crate::git;
use crate::lua::{Config, Shared};
use crate::output;
use crate::state::{self, Classes};
use crate::utils::{self, Workspace};

use super::super::constants::{PREVIEW_LIMIT, YES_FLAGS};

#[derive(Debug, Args)]
pub struct RmArgs {
    #[arg(value_name = "PATH", required = true)]
    pub paths: Vec<String>,
    #[arg(short, long, help = "Stop managing them without asking first")]
    pub yes: bool,
    #[arg(short = 'n', long, help = "Report what would happen, touching nothing")]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Detached {
    Restored,
    Untouched,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Plan {
    Keep,
    Copy,
    Relink,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Counts {
    restored: u32,
    untouched: u32,
}

pub fn rm_cmd(args: RmArgs) -> Result<()> {
    let Workspace {
        config: shared,
        home,
        repo,
    } = utils::workspace("rm")?;
    let config = utils::configured("rm", &shared)?;

    let entries = plan(&home, &repo, &args.paths)?;
    if entries.is_empty() {
        output::note("nothing to remove");
        return Ok(());
    }

    let classes = classes(&entries)?;
    if args.dry_run {
        return foresee(&home, &repo, &entries, &classes, &shared);
    }

    let lock = config.crypt_lock();
    let mut identity = config.crypt_identity(&home);

    if !args.yes && !confirmed(&repo, &entries)? {
        output::warn("aborted");
        return Ok(());
    }

    let mut backup = match config.backup() {
        true => Some(Backup::open(
            "rm",
            &home,
            config.backup_dir(),
            config.retention(),
        )?),
        false => None,
    };

    let mut counts = Counts::default();
    for entry in &entries {
        let detached = match entry {
            Entry::File(file) => {
                vec![detach_file(
                    &config,
                    lock,
                    &mut identity,
                    &home,
                    &repo,
                    file,
                    &mut backup,
                )?]
            }
            template => detach_template(&home, &repo, template, &classes, &mut backup, &shared)?,
        };
        counts.record(&detached);
    }
    let removed = removed(&entries);
    git::unstage("rm", &repo, &removed)?;

    let automatic = utils::automatic(&config, &repo, &removed);
    git::auto("rm", &repo, automatic.commits, automatic.pushes)?;

    output::note(summary("stopped managing", &entries, &counts));
    if let Some(backup) = backup.as_ref() {
        backup.finish()?;
    }

    Ok(())
}

fn removed(entries: &[Entry]) -> Vec<PathBuf> {
    entries
        .iter()
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

fn foresee(
    home: &Path,
    repo: &Path,
    entries: &[Entry],
    classes: &Classes,
    shared: &Shared,
) -> Result<()> {
    output::line(preview(repo, entries, entries.len()));

    let mut counts = Counts::default();
    for entry in entries {
        let detached = match entry {
            Entry::File(file) => vec![foresee_file(home, repo, file)?],
            template => foresee_template(home, repo, template, classes, shared)?,
        };
        counts.record(&detached);
    }

    output::note(summary("would stop managing", entries, &counts));

    Ok(())
}

fn confirmed(repo: &Path, entries: &[Entry]) -> Result<bool> {
    if entries.len() < 2 {
        return Ok(true);
    }

    output::line(preview(repo, entries, PREVIEW_LIMIT));
    output::confirm(
        "rm",
        &format!("Stop managing {}?", what(entries)),
        YES_FLAGS,
    )
}

fn preview(repo: &Path, entries: &[Entry], limit: usize) -> String {
    let mut lines: Vec<String> = entries
        .iter()
        .take(limit)
        .map(|entry| format!("  {}", utils::relative(repo, entry.path()).display()))
        .collect();

    if entries.len() > limit {
        lines.push(format!("  ... and {} more", entries.len() - limit));
    }

    lines.join("\n")
}

fn summary(verb: &str, entries: &[Entry], counts: &Counts) -> String {
    format!(
        "{verb} {} ({} restored, {} left untouched)",
        what(entries),
        counts.restored,
        counts.untouched
    )
}

fn what(entries: &[Entry]) -> String {
    let templates = entries
        .iter()
        .filter(|entry| !matches!(entry, Entry::File(_)))
        .count();
    let files = entries.len() - templates;

    let mut counted = Vec::new();
    if files > 0 || templates == 0 {
        counted.push(format!("{files} file(s)"));
    }
    if templates > 0 {
        counted.push(format!("{templates} template(s)"));
    }

    counted.join(" and ")
}

fn plan(home: &Path, repo: &Path, args: &[String]) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();
    for arg in args {
        let root = utils::managed_path("rm", home, repo, arg)?;
        entries.extend(files::collect_entries("rm", &root)?);
    }
    entries.sort_by(|left, right| left.path().cmp(right.path()));
    entries.dedup();

    Ok(entries)
}

fn classes(entries: &[Entry]) -> Result<Classes> {
    if !entries
        .iter()
        .any(|entry| matches!(entry, Entry::Template(_)))
    {
        return Ok(Classes::default());
    }

    Ok(state::load()?.classes().clone())
}

fn detach_file(
    config: &Config,
    lock: crypt::Lock,
    identity: &mut crypt::Identity,
    home: &Path,
    repo: &Path,
    file: &Path,
    backup: &mut Option<Backup>,
) -> Result<Detached> {
    let split = crypt::split(utils::relative(repo, file));
    let dest = detach_target(home, repo, file, &split)?;
    let detached = match &split {
        Some((stripped, backend)) => {
            detach_encrypted(config, *backend, lock, identity, stripped, file, &dest)?
        }
        None => detach(file, &dest, backup)?,
    };

    if let Some(backup) = backup.as_mut() {
        backup.save(file)?;
    }
    std::fs::remove_file(file)
        .with_context(|| format!("rm: failed to remove {}", file.display()))?;
    prune_parents(repo, file)?;

    Ok(detached)
}

fn detach_template(
    home: &Path,
    repo: &Path,
    entry: &Entry,
    classes: &Classes,
    backup: &mut Option<Backup>,
    shared: &Shared,
) -> Result<Vec<Detached>> {
    let template = entry.path();

    let mut detached = Vec::new();
    for dest in produced(home, repo, entry, classes, shared)? {
        detached.push(match link_into(template, &dest)? {
            Some(source) => detach(&source, &dest, backup)?,
            None => Detached::Untouched,
        });
    }

    for file in files::collect_files("rm", template)? {
        if let Some(backup) = backup.as_mut() {
            backup.save(&file)?;
        }
    }
    remove_template(template)?;
    prune_parents(repo, template)?;

    Ok(detached)
}

fn foresee_file(home: &Path, repo: &Path, file: &Path) -> Result<Detached> {
    let split = crypt::split(utils::relative(repo, file));
    let dest = detach_target(home, repo, file, &split)?;

    let Some(_) = &split else {
        return Ok(decide(file, &dest)?.detached());
    };

    Ok(match metadata(&dest)?.is_some() {
        true => Detached::Untouched,
        false => Detached::Restored,
    })
}

fn foresee_template(
    home: &Path,
    repo: &Path,
    entry: &Entry,
    classes: &Classes,
    shared: &Shared,
) -> Result<Vec<Detached>> {
    let template = entry.path();

    produced(home, repo, entry, classes, shared)?
        .iter()
        .map(|dest| {
            Ok(match link_into(template, dest)? {
                Some(_) => Detached::Restored,
                None => Detached::Untouched,
            })
        })
        .collect()
}

fn produced(
    home: &Path,
    repo: &Path,
    entry: &Entry,
    classes: &Classes,
    shared: &Shared,
) -> Result<Vec<PathBuf>> {
    let mirrored = utils::system_path(home, repo, &entry.target())?;

    let Entry::Template(_) = entry else {
        return Ok(vec![mirrored]);
    };

    match utils::outputs("rm", home, repo, entry, classes, shared) {
        Ok(outputs) => Ok(outputs
            .iter()
            .map(|output| output.dest().to_path_buf())
            .collect()),
        Err(err) => {
            output::warn(format!(
                "{} did not resolve, only {} was inspected: {err:#}",
                entry.path().display(),
                mirrored.display()
            ));

            Ok(vec![mirrored])
        }
    }
}

fn link_into(template: &Path, dest: &Path) -> Result<Option<PathBuf>> {
    let Some(meta) = metadata(dest)? else {
        return Ok(None);
    };
    if !meta.file_type().is_symlink() {
        return Ok(None);
    }

    let target = std::fs::read_link(dest)
        .with_context(|| format!("rm: failed to read {}", dest.display()))?;

    Ok(target.starts_with(template).then_some(target))
}

fn remove_template(template: &Path) -> Result<()> {
    let removed = match template.is_dir() {
        true => std::fs::remove_dir_all(template),
        false => std::fs::remove_file(template),
    };

    removed.with_context(|| format!("rm: failed to remove {}", template.display()))
}

fn detach_target(
    home: &Path,
    repo: &Path,
    file: &Path,
    split: &Option<(PathBuf, crypt::Backend)>,
) -> Result<PathBuf> {
    match split {
        Some((stripped, _)) => utils::system_path(home, repo, &repo.join(stripped)),
        None => utils::system_path(home, repo, file),
    }
}

fn detach_encrypted(
    config: &Config,
    backend: crypt::Backend,
    lock: crypt::Lock,
    identity: &mut crypt::Identity,
    stripped: &Path,
    source: &Path,
    dest: &Path,
) -> Result<Detached> {
    if metadata(dest)?.is_some() {
        return Ok(Detached::Untouched);
    }

    let contents = crypt::decrypt("rm", backend, lock, identity.path("rm")?, source)
        .with_context(|| format!("rm: failed to decrypt {}", source.display()))?;

    if utils::is_root(stripped) {
        crypt::place_system(
            "rm",
            files::ConflictPolicy::Overwrite,
            &contents,
            dest,
            config.mode(stripped),
            config.owner(stripped),
        )?;
        return Ok(Detached::Restored);
    }

    crypt::place("rm", files::ConflictPolicy::Overwrite, &contents, dest)?;
    Ok(Detached::Restored)
}

fn detach(source: &Path, dest: &Path, backup: &mut Option<Backup>) -> Result<Detached> {
    let plan = decide(source, dest)?;

    if plan == Plan::Relink {
        if let Some(backup) = backup.as_mut() {
            backup.save(dest)?;
        }
        std::fs::remove_file(dest)
            .with_context(|| format!("rm: failed to remove {}", dest.display()))?;
    }
    if plan != Plan::Keep {
        restore(source, dest)?;
    }

    Ok(plan.detached())
}

fn decide(source: &Path, dest: &Path) -> Result<Plan> {
    if metadata(source)?.is_some_and(|meta| meta.file_type().is_symlink()) {
        return Ok(Plan::Keep);
    }

    let Some(meta) = metadata(dest)? else {
        return Ok(Plan::Copy);
    };

    if !meta.file_type().is_symlink() || !points_at(dest, source)? {
        return Ok(Plan::Keep);
    }

    Ok(Plan::Relink)
}

impl Plan {
    fn detached(self) -> Detached {
        match self {
            Self::Keep => Detached::Untouched,
            Self::Copy | Self::Relink => Detached::Restored,
        }
    }
}

impl Counts {
    fn record(&mut self, detached: &[Detached]) {
        for one in detached {
            match one {
                Detached::Restored => self.restored += 1,
                Detached::Untouched => self.untouched += 1,
            }
        }
    }
}

fn restore(source: &Path, dest: &Path) -> Result<()> {
    match restore_plain(source, dest) {
        Err(err) if files::permission_denied(&err) => files::escalate_entry("rm", source, dest),
        other => other,
    }
}

fn restore_plain(source: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("rm: failed to create {}", parent.display()))?;
    }

    std::fs::copy(source, dest).map(|_| ()).with_context(|| {
        format!(
            "rm: failed to restore {} from {}",
            dest.display(),
            source.display()
        )
    })
}

fn prune_parents(repo: &Path, file: &Path) -> Result<()> {
    let mut current = file.parent();
    while let Some(dir) = current.filter(|dir| *dir != repo && dir.starts_with(repo)) {
        if !is_empty(dir)? {
            return Ok(());
        }
        std::fs::remove_dir(dir)
            .with_context(|| format!("rm: failed to remove {}", dir.display()))?;
        current = dir.parent();
    }
    Ok(())
}

fn is_empty(dir: &Path) -> Result<bool> {
    let mut entries =
        std::fs::read_dir(dir).with_context(|| format!("rm: failed to read {}", dir.display()))?;
    Ok(entries.next().is_none())
}

fn metadata(path: &Path) -> Result<Option<Metadata>> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) => Ok(Some(meta)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("rm: failed to inspect {}", path.display())),
    }
}

fn points_at(link: &Path, target: &Path) -> Result<bool> {
    let read = std::fs::read_link(link)
        .with_context(|| format!("rm: failed to read {}", link.display()))?;
    Ok(read == target)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    fn configuration() -> crate::lua::Shared {
        Arc::new(Mutex::new(Config::default()))
    }

    use super::*;

    fn write(path: &Path, contents: &str) {
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn a_plan_says_what_detaching_would_do_without_doing_it() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");

        assert_eq!(decide(&source, &dest).unwrap(), Plan::Copy);
        assert!(!dest.exists());

        std::os::unix::fs::symlink(&source, &dest).unwrap();
        assert_eq!(decide(&source, &dest).unwrap(), Plan::Relink);
        assert!(std::fs::symlink_metadata(&dest).unwrap().is_symlink());
    }

    #[test]
    fn detach_restores_a_missing_system_file() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("nested").join("dest");
        write(&source, "data");

        assert_eq!(
            detach(&source, &dest, &mut None).unwrap(),
            Detached::Restored
        );
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
    }

    #[test]
    fn detach_materializes_a_symlink_pointing_into_the_repository() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        std::os::unix::fs::symlink(&source, &dest).unwrap();

        assert_eq!(
            detach(&source, &dest, &mut None).unwrap(),
            Detached::Restored
        );

        let kind = std::fs::symlink_metadata(&dest).unwrap().file_type();
        assert!(!kind.is_symlink());
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
    }

    #[test]
    fn detach_saves_the_symlink_it_replaces() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let source = dir.path().join("source");
        let dest = home.join(".zshrc");
        let saved = dir.path().join("backup");
        std::fs::create_dir_all(&home).unwrap();
        write(&source, "data");
        std::os::unix::fs::symlink(&source, &dest).unwrap();

        let mut backup = Some(Backup::at("rm", &home, saved.clone()));
        assert_eq!(
            detach(&source, &dest, &mut backup).unwrap(),
            Detached::Restored
        );

        assert_eq!(
            std::fs::read_link(saved.join("home/.zshrc")).unwrap(),
            source
        );
        assert_eq!(backup.unwrap().saved(), 1);
    }

    #[test]
    fn detach_leaves_a_hard_linked_system_file_alone() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        std::fs::hard_link(&source, &dest).unwrap();

        assert_eq!(
            detach(&source, &dest, &mut None).unwrap(),
            Detached::Untouched
        );

        std::fs::remove_file(&source).unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
    }

    #[test]
    fn detach_keeps_a_diverging_system_file() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "repo");
        write(&dest, "system");

        assert_eq!(
            detach(&source, &dest, &mut None).unwrap(),
            Detached::Untouched
        );
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "system");
    }

    #[test]
    fn prune_parents_removes_empty_directories_up_to_the_repository() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        let nested = repo.join(".config").join("nvim");
        std::fs::create_dir_all(&nested).unwrap();

        prune_parents(&repo, &nested.join("init.lua")).unwrap();

        assert!(!repo.join(".config").exists());
        assert!(repo.is_dir());
    }

    #[test]
    fn plan_collects_every_file_below_a_managed_directory() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = repo.join("home/.config/nvim");
        std::fs::create_dir_all(nvim.join("lua")).unwrap();
        write(&nvim.join("init.lua"), "init");
        write(&nvim.join("lua").join("plugins.lua"), "plugins");

        let arg = home
            .join(".config")
            .join("nvim")
            .to_string_lossy()
            .into_owned();
        let entries = plan(&home, &repo, &[arg]).unwrap();

        assert_eq!(
            entries,
            vec![
                Entry::File(nvim.join("init.lua")),
                Entry::File(nvim.join("lua").join("plugins.lua")),
            ]
        );
    }

    #[test]
    fn plan_keeps_a_template_whole() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let template = repo.join("home/.zshrc.luadot");
        std::fs::create_dir_all(&template).unwrap();
        write(&template.join("luadot.lua"), "return \"\"\n");
        write(&template.join("laptop.zsh"), "laptop");

        let arg = home.join(".zshrc").to_string_lossy().into_owned();
        let entries = plan(&home, &repo, &[arg]).unwrap();

        assert_eq!(entries, vec![Entry::Template(template)]);
    }

    #[test]
    fn a_template_goes_away_and_leaves_what_it_generated_behind() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let template = repo.join("home/.zshrc.luadot");
        std::fs::create_dir_all(&template).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        write(&template.join("luadot.lua"), r#"return "generated\n""#);
        write(&home.join(".zshrc"), "generated\n");

        let detached = detach_template(
            &home,
            &repo,
            &Entry::Template(template.clone()),
            &Classes::default(),
            &mut None,
            &configuration(),
        )
        .unwrap();

        assert_eq!(detached, vec![Detached::Untouched]);
        assert!(!template.exists());
        assert_eq!(
            std::fs::read_to_string(home.join(".zshrc")).unwrap(),
            "generated\n"
        );
    }

    #[test]
    fn a_link_into_the_template_becomes_a_file_of_its_own() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let template = repo.join("home/.zshrc.luadot");
        std::fs::create_dir_all(&template).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        write(&template.join("laptop.zsh"), "laptop\n");
        write(
            &template.join("luadot.lua"),
            r#"return { content = ld.alt.file("laptop.zsh"), link = "symbolic" }"#,
        );
        std::os::unix::fs::symlink(template.join("laptop.zsh"), home.join(".zshrc")).unwrap();

        let detached = detach_template(
            &home,
            &repo,
            &Entry::Template(template.clone()),
            &Classes::default(),
            &mut None,
            &configuration(),
        )
        .unwrap();

        assert_eq!(detached, vec![Detached::Restored]);
        assert!(!template.exists());
        assert!(
            !std::fs::symlink_metadata(home.join(".zshrc"))
                .unwrap()
                .is_symlink()
        );
        assert_eq!(
            std::fs::read_to_string(home.join(".zshrc")).unwrap(),
            "laptop\n"
        );
    }

    #[test]
    fn a_standalone_template_goes_away_on_its_own() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let template = repo.join("home/.zprofile.luadot");
        std::fs::create_dir_all(template.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        write(&template, "export HOST=1\n");
        write(&home.join(".zprofile"), "export HOST=1\n");

        let detached = detach_template(
            &home,
            &repo,
            &Entry::Standalone(template.clone()),
            &Classes::default(),
            &mut None,
            &configuration(),
        )
        .unwrap();

        assert_eq!(detached, vec![Detached::Untouched]);
        assert!(!template.exists());
        assert_eq!(
            std::fs::read_to_string(home.join(".zprofile")).unwrap(),
            "export HOST=1\n"
        );
    }
}
