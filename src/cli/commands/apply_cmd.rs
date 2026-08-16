use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::files::{self, Entry, SyncOutcome};
use crate::lua;
use crate::output;
use crate::utils::{self, Backup};

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
        .filter(|entry| !config.is_ignored(utils::relative(&repo, &entry.target())))
        .filter_map(|entry| match entry {
            Entry::File(file) => Some(file),
            Entry::Template(_) | Entry::Standalone(_) => None,
        })
        .collect();
    if files.is_empty() {
        output::note("nothing to apply");
        return Ok(());
    }

    let mut backup = match args.dry_run || !config.backup() {
        true => None,
        false => Some(Backup::open("apply", &home)?),
    };

    let mut created = 0u32;
    let mut replaced = 0u32;
    let mut unchanged = 0u32;
    let mut skipped = 0u32;
    for file in &files {
        let relative = utils::relative(&repo, file);
        let dest = utils::system_path(&home, &repo, file)?;
        let mode = config.link_mode(relative);
        let policy = config.conflict_policy(relative);

        let status = files::file_status(mode, file, &dest)
            .with_context(|| format!("apply: failed to inspect {}", dest.display()))?;
        let predicted = files::predict(policy, status, &dest)
            .with_context(|| format!("apply: failed to apply {}", dest.display()))?;

        let outcome = match args.dry_run {
            true => {
                utils::preview(predicted, relative.display());
                predicted
            }
            false => {
                if predicted == SyncOutcome::Replaced
                    && let Some(backup) = backup.as_mut()
                {
                    backup.save(&dest)?;
                }
                files::sync_file(policy, mode, file, &dest)
                    .with_context(|| format!("apply: failed to apply {}", dest.display()))?
            }
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
    if let Some(backup) = backup.as_ref() {
        backup.report();
    }

    Ok(())
}
