use std::fs::Metadata;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
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

    let entries = plan(&home, &repo, &args.paths)?;
    if entries.is_empty() {
        output::note("nothing to remove");
        return Ok(());
    }

    let wholes = whole_roots(&shared, &home, &repo, &args.paths, &entries)?;
    let classes = classes(&entries)?;
    if args.dry_run {
        for (system, _) in &wholes {
            output::note(format!(
                "would restore {} in place of its link",
                system.display()
            ));
        }
        return foresee(&home, &repo, &entries, &classes, &shared);
    }

    let (lock, mut identity) = {
        let config = utils::configured("rm", &shared)?;
        (config.crypt_lock(), config.crypt_identity(&home))
    };

    if !args.yes && !confirmed(&repo, &entries)? {
        output::warn("aborted");
        return Ok(());
    }

    let mut backup = {
        let config = utils::configured("rm", &shared)?;
        match config.backup() {
            true => Some(Backup::open("rm", config.backup_dir(), config.retention())?),
            false => None,
        }
    };

    for (system, stored) in &wholes {
        materialize(system, stored, &mut backup)?;
    }

    let mut counts = Counts::default();
    for entry in &entries {
        let detached = match entry {
            Entry::File(file) => {
                let config = utils::configured("rm", &shared)?;
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

    let automatic = {
        let config = utils::configured("rm", &shared)?;
        utils::automatic(&config, &repo, &removed)
    };
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

fn whole_roots(
    shared: &Shared,
    home: &Path,
    repo: &Path,
    args: &[String],
    entries: &[Entry],
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let config = utils::configured("rm", shared)?;

    let mut roots: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let Entry::File(file) = entry else {
            continue;
        };
        let logical = crypt::logical(utils::relative(repo, file));
        let Some(root) = config.unit_root(&logical) else {
            continue;
        };
        if !roots.contains(&root) {
            roots.push(root);
        }
    }
    if roots.is_empty() {
        return Ok(Vec::new());
    }

    let mut asked: Vec<PathBuf> = Vec::new();
    for arg in args {
        asked.push(utils::managed_path("rm", home, repo, arg)?);
    }

    let mut wholes = Vec::new();
    for root in roots {
        let stored = repo.join(&root);
        if !asked.iter().any(|path| stored.starts_with(path)) {
            bail!(
                "rm: {} is placed whole; run `luadot rm {}`",
                root.display(),
                home.join(&root).display()
            );
        }
        let system = home.join(&root);
        if files::link_at("rm", &system)?.as_deref() == Some(stored.as_path()) {
            wholes.push((system, stored));
        }
    }

    Ok(wholes)
}

fn materialize(system: &Path, stored: &Path, backup: &mut Option<Backup>) -> Result<()> {
    if let Some(backup) = backup.as_mut() {
        backup.save(system)?;
    }
    std::fs::remove_file(system)
        .with_context(|| format!("rm: failed to remove {}", system.display()))?;
    files::copy_tree("rm", stored, system)?;
    output::note(format!(
        "restored {} in place of its link",
        system.display()
    ));

    Ok(())
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
    files::prune_parents("rm", repo, file)?;

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
    files::prune_parents("rm", repo, template)?;

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

    crypt::place(
        "rm",
        files::ConflictPolicy::Overwrite,
        config.placement(stripped),
        &contents,
        dest,
    )?;
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

    if metadata(dest)?.is_none() {
        return Ok(Plan::Copy);
    }
    if files::link_at("rm", dest)?.as_deref() != Some(source) {
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

fn metadata(path: &Path) -> Result<Option<Metadata>> {
    files::metadata("rm", path)
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
    fn detach_materializes_a_symlink() {
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

        let mut backup = Some(Backup::at("rm", saved.clone()));
        assert_eq!(
            detach(&source, &dest, &mut backup).unwrap(),
            Detached::Restored
        );

        assert_eq!(
            std::fs::read_link(saved.join(dest.strip_prefix("/").unwrap())).unwrap(),
            source
        );
        assert_eq!(backup.unwrap().saved(), 1);
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
    fn plan_collects_a_whole_directory() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = repo.join(".config/nvim");
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
    fn a_template_leaves_its_output() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let template = repo.join(".zshrc.luadot");
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
    fn a_template_link_becomes_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let template = repo.join(".zshrc.luadot");
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
}
