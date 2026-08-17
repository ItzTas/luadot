use anyhow::{Context, Result};
use clap::Args;

use crate::crypt;
use crate::files::{self, FileStatus};
use crate::output::{self, Tone};
use crate::utils::{self, Workspace};

use super::super::constants::STATUS_LABELS;

#[derive(Debug, Args)]
pub struct StatusArgs {
    #[arg(value_name = "PATH")]
    pub path: Option<String>,
}

pub fn status_cmd(args: StatusArgs) -> Result<()> {
    let Workspace { config, home, repo } = utils::workspace("status")?;

    let root = utils::managed_root("status", &home, &repo, args.path.as_deref())?;

    let files = utils::managed_files("status", &repo, &root, |relative| {
        config.is_ignored(&crypt::logical(relative))
    })?;
    if files.is_empty() {
        output::note("nothing is managed");
        return Ok(());
    }

    let mut synced = 0u32;
    let mut missing = 0u32;
    let mut unlinked = 0u32;
    let mut differs = 0u32;
    let mut unreadable = 0u32;
    let identity = config
        .crypt_identity()
        .map(|path| utils::expand(&home, path));
    for file in &files {
        let relative = utils::relative(&repo, file);
        let split = crypt::split(relative);
        let logical = split
            .as_ref()
            .map(|(stripped, _)| stripped.as_path())
            .unwrap_or(relative);
        let dest = utils::system_path(&home, &repo, &repo.join(logical))?;
        let status = match split {
            Some((_, backend)) => {
                crypt::status("status", backend, identity.as_deref(), file, &dest)
            }
            None => match utils::is_root(relative) {
                true => files::inspect_system(file, &dest, config.mode(relative)),
                false => files::file_status(config.link_mode(relative), file, &dest),
            },
        }
        .with_context(|| format!("status: failed to inspect {}", dest.display()))?;
        match status {
            FileStatus::Synced => {
                synced += 1;
                continue;
            }
            FileStatus::Missing => missing += 1,
            FileStatus::Unlinked => unlinked += 1,
            FileStatus::Differs => differs += 1,
            FileStatus::Unreadable => unreadable += 1,
        }
        let (tone, label) = display(status);
        output::entry(tone, label, relative.display());
    }

    let mut counts =
        format!("{synced} synced, {missing} missing, {unlinked} unlinked, {differs} differs");
    if unreadable > 0 {
        counts.push_str(&format!(", {unreadable} unreadable"));
    }
    output::note(format!("{} managed file(s) ({counts})", files.len()));

    Ok(())
}

fn display(status: FileStatus) -> (Tone, &'static str) {
    STATUS_LABELS
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
            FileStatus::Unreadable,
        ] {
            let (_, label) = display(status);
            assert!(!label.is_empty());
        }
    }

    #[test]
    fn labels_fit_the_printed_column() {
        for (_, text, _) in STATUS_LABELS {
            assert!(text.len() < 11);
        }
    }
}
