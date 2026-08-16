use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::files::{self, Entry, FileStatus};
use crate::lua;
use crate::output::{self, Tone};
use crate::utils;

const LABELS: [(FileStatus, &str, Tone); 4] = [
    (FileStatus::Synced, "synced", Tone::Good),
    (FileStatus::Missing, "missing", Tone::Warning),
    (FileStatus::Unlinked, "unlinked", Tone::Warning),
    (FileStatus::Differs, "differs", Tone::Bad),
];

#[derive(Debug, Args)]
pub struct StatusArgs {
    #[arg(value_name = "PATH")]
    pub path: Option<String>,
}

pub fn status_cmd(args: StatusArgs) -> Result<()> {
    let config = lua::load_config()?;
    let repo = utils::require_repo("status", config.repo_dir())?;

    let home = utils::home_dir()?;

    let root = match args.path.as_deref() {
        Some(path) => utils::managed_path("status", &home, &repo, path)?,
        None => repo.clone(),
    };

    let files: Vec<PathBuf> = files::collect_entries("status", &root)?
        .into_iter()
        .filter(|entry| !config.is_ignored(utils::relative(&repo, &entry.target())))
        .filter_map(|entry| match entry {
            Entry::File(file) => Some(file),
            Entry::Template(_) | Entry::Standalone(_) => None,
        })
        .collect();
    if files.is_empty() {
        output::note("nothing is managed");
        return Ok(());
    }

    let mut synced = 0u32;
    let mut missing = 0u32;
    let mut unlinked = 0u32;
    let mut differs = 0u32;
    for file in &files {
        let relative = utils::relative(&repo, file);
        let dest = utils::system_path(&home, &repo, file)?;
        let status = files::file_status(config.link_mode(relative), file, &dest)
            .with_context(|| format!("status: failed to inspect {}", dest.display()))?;
        match status {
            FileStatus::Synced => {
                synced += 1;
                continue;
            }
            FileStatus::Missing => missing += 1,
            FileStatus::Unlinked => unlinked += 1,
            FileStatus::Differs => differs += 1,
        }
        let (tone, label) = display(status);
        output::entry(tone, label, relative.display());
    }

    output::note(format!(
        "{} managed file(s) ({synced} synced, {missing} missing, {unlinked} unlinked, {differs} differs)",
        files.len()
    ));

    Ok(())
}

fn display(status: FileStatus) -> (Tone, &'static str) {
    LABELS
        .iter()
        .find(|(kind, _, _)| *kind == status)
        .map(|(_, label, tone)| (*tone, *label))
        .unwrap_or((Tone::Muted, ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_label() {
        for status in [
            FileStatus::Synced,
            FileStatus::Missing,
            FileStatus::Unlinked,
            FileStatus::Differs,
        ] {
            let (_, label) = display(status);
            assert!(!label.is_empty());
        }
    }

    #[test]
    fn labels_fit_the_printed_column() {
        for (_, text, _) in LABELS {
            assert!(text.len() < 9);
        }
    }
}
