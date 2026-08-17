use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::files::{self, Entry, SyncOutcome};
use crate::lua::{self, Config};
use crate::output;
use crate::utils::{self, Backup, Hooks};

#[derive(Debug, Args)]
pub struct ApplyArgs {
    #[arg(value_name = "PATH")]
    pub path: Option<String>,
    #[arg(
        short = 'n',
        long,
        help = "Report what would change, touching nothing and taking no backup"
    )]
    pub dry_run: bool,
}

#[derive(Debug, Default)]
struct Run {
    dry_run: bool,
    backup: Option<Backup>,
    hooks: Hooks,
}

pub fn apply_cmd(args: ApplyArgs) -> Result<()> {
    let config = lua::load_config()?;
    let repo = utils::require_repo("apply", config.repo_dir())?;

    let home = utils::home_dir()?;

    let root = match args.path.as_deref() {
        Some(path) => utils::managed_path("apply", &home, &repo, path)?,
        None => repo.clone(),
    };

    let files: Vec<PathBuf> = files::collect_entries("apply", &root)?
        .into_iter()
        .filter(|entry| {
            let target = entry.target();
            let relative = utils::relative(&repo, &target);
            utils::is_managed(relative) && !config.is_ignored(relative)
        })
        .filter_map(|entry| match entry {
            Entry::File(file) => Some(file),
            Entry::Template(_) | Entry::Standalone(_) => None,
        })
        .collect();
    if files.is_empty() {
        output::note("nothing to apply");
        return Ok(());
    }

    let mut run = Run {
        dry_run: args.dry_run,
        backup: match args.dry_run || !config.backup() {
            true => None,
            false => Some(Backup::open(
                "apply",
                &home,
                config.backup_dir(),
                config.backup_keep(),
            )?),
        },
        hooks: Hooks::new(args.dry_run),
    };

    let mut created = 0u32;
    let mut replaced = 0u32;
    let mut unchanged = 0u32;
    let mut skipped = 0u32;
    for file in &files {
        let relative = utils::relative(&repo, file);
        let dest = utils::system_path(&home, &repo, file)?;

        let outcome = match utils::is_root(relative) {
            true => place_root(&config, relative, file, &dest, &mut run)?,
            false => place_home(&config, relative, file, &dest, &mut run)?,
        };

        match outcome {
            SyncOutcome::Created => created += 1,
            SyncOutcome::Replaced => replaced += 1,
            SyncOutcome::AlreadySynced => unchanged += 1,
            SyncOutcome::Skipped => skipped += 1,
        }
    }

    output::note(format!(
        "{} {} file(s) ({created} created, {replaced} replaced, {unchanged} unchanged, {skipped} skipped)",
        match args.dry_run {
            true => "would apply",
            false => "applied",
        },
        files.len()
    ));
    if let Some(backup) = run.backup.as_ref() {
        backup.finish()?;
    }
    run.hooks.finish("apply")?;

    Ok(())
}

fn place_home(
    config: &Config,
    relative: &Path,
    file: &Path,
    dest: &Path,
    run: &mut Run,
) -> Result<SyncOutcome> {
    let mode = config.link_mode(relative);
    let policy = config.conflict_policy(relative);

    let status = files::file_status(mode, file, dest)
        .with_context(|| format!("apply: failed to inspect {}", dest.display()))?;
    let predicted = files::predict(policy, status, dest)
        .with_context(|| format!("apply: failed to apply {}", dest.display()))?;

    if run.dry_run {
        utils::preview(predicted, relative.display());
        run.hooks.record(predicted, config.on_change(relative));
        return Ok(predicted);
    }

    if predicted == SyncOutcome::Replaced
        && let Some(backup) = run.backup.as_mut()
    {
        backup.save(dest)?;
    }
    let outcome = files::sync_file(policy, mode, file, dest)
        .with_context(|| format!("apply: failed to apply {}", dest.display()))?;

    run.hooks.record(outcome, config.on_change(relative));

    Ok(outcome)
}

fn place_root(
    config: &Config,
    relative: &Path,
    file: &Path,
    dest: &Path,
    run: &mut Run,
) -> Result<SyncOutcome> {
    let policy = config.conflict_policy(relative);
    let mode = config.mode(relative);

    let status = files::escalated_status(file, dest, mode)
        .with_context(|| format!("apply: failed to inspect {}", dest.display()))?;
    let predicted = files::predict(policy, status, dest)
        .with_context(|| format!("apply: failed to apply {}", dest.display()))?;

    if run.dry_run {
        utils::preview(predicted, relative.display());
        run.hooks.record(predicted, config.on_change(relative));
        return Ok(predicted);
    }

    if predicted == SyncOutcome::Replaced
        && let Some(backup) = run.backup.as_mut()
    {
        backup.save(dest)?;
    }
    let outcome = files::sync_system(policy, file, dest, mode, config.owner(relative))
        .with_context(|| format!("apply: failed to apply {}", dest.display()))?;

    run.hooks.record(outcome, config.on_change(relative));

    Ok(outcome)
}
