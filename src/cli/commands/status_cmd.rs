use std::fmt::{self, Display};
use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;

use crate::crypt;
use crate::files::{self, Entry, FileStatus};
use crate::lua::Config;
use crate::output::{self, Tone};
use crate::state::{self, Classes};
use crate::utils::{self, Workspace};

use super::super::constants::STATUS_LABELS;

#[derive(Debug, Args)]
pub struct StatusArgs {
    #[arg(value_name = "PATH")]
    pub path: Option<String>,
    #[arg(
        short,
        long,
        help = "Resolve the templates and report the files they produce"
    )]
    pub templates: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Counts([u32; STATUS_LABELS.len()]);

pub fn status_cmd(args: StatusArgs) -> Result<()> {
    let Workspace { config, home, repo } = utils::workspace("status")?;

    let root = utils::managed_root("status", &home, &repo, args.path.as_deref())?;

    let (files, templates): (Vec<Entry>, Vec<Entry>) =
        utils::managed_entries("status", &repo, &root, |relative| {
            config.is_ignored(&crypt::logical(relative))
        })?
        .into_iter()
        .partition(|entry| matches!(entry, Entry::File(_)));

    if files.is_empty() && templates.is_empty() {
        output::note("nothing is managed");
        return Ok(());
    }

    if !files.is_empty() {
        let counts = report_files(&config, &home, &repo, &files)?;
        output::note(format!("{} managed file(s) ({counts})", files.len()));
    }

    if templates.is_empty() {
        return Ok(());
    }
    if !args.templates {
        output::note(format!(
            "{} template(s) skipped (run with --templates)",
            templates.len()
        ));
        return Ok(());
    }

    let classes = state::load()?.classes().clone();
    let (produced, counts) = report_templates(&config, &home, &repo, &templates, &classes)?;
    output::note(format!(
        "{} template(s) into {produced} file(s) ({counts})",
        templates.len()
    ));

    Ok(())
}

fn report_files(config: &Config, home: &Path, repo: &Path, files: &[Entry]) -> Result<Counts> {
    let identity = config
        .crypt_identity()
        .map(|path| utils::expand(home, path));

    let mut counts = Counts::default();
    for file in files.iter().map(Entry::path) {
        let relative = utils::relative(repo, file);
        let split = crypt::split(relative);
        let logical = split
            .as_ref()
            .map(|(stripped, _)| stripped.as_path())
            .unwrap_or(relative);
        let dest = utils::system_path(home, repo, &repo.join(logical))?;
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

        counts.record(status);
        report(status, relative.display());
    }

    Ok(counts)
}

fn report_templates(
    config: &Config,
    home: &Path,
    repo: &Path,
    templates: &[Entry],
    classes: &Classes,
) -> Result<(usize, Counts)> {
    let mut produced = 0usize;
    let mut counts = Counts::default();
    for entry in templates {
        for output in utils::outputs("status", home, repo, entry, classes)? {
            let status = utils::output_status("status", config, home, &output)?;
            let relative = utils::output_relative("status", home, &output)?;

            produced += 1;
            counts.record(status);
            report(status, relative.display());
        }
    }

    Ok((produced, counts))
}

fn report(status: FileStatus, path: impl Display) {
    if status == FileStatus::Synced {
        return;
    }

    let (tone, label) = display(status);
    output::entry(tone, label, path);
}

fn display(status: FileStatus) -> (Tone, &'static str) {
    STATUS_LABELS
        .iter()
        .find(|(kind, _, _)| *kind == status)
        .map(|(_, label, tone)| (*tone, *label))
        .unwrap_or((Tone::Muted, ""))
}

impl Counts {
    fn record(&mut self, status: FileStatus) {
        let Some(index) = STATUS_LABELS
            .iter()
            .position(|(kind, _, _)| *kind == status)
        else {
            return;
        };

        self.0[index] += 1;
    }
}

impl Display for Counts {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let counted: Vec<String> = STATUS_LABELS
            .iter()
            .zip(self.0)
            .filter(|((kind, _, _), count)| *kind != FileStatus::Unreadable || *count > 0)
            .map(|((_, label, _), count)| format!("{count} {label}"))
            .collect();

        formatter.write_str(&counted.join(", "))
    }
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

    #[test]
    fn the_counts_name_every_state_a_file_can_be_in() {
        let mut counts = Counts::default();
        counts.record(FileStatus::Synced);
        counts.record(FileStatus::Synced);
        counts.record(FileStatus::Differs);

        assert_eq!(
            counts.to_string(),
            "2 synced, 0 missing, 0 unlinked, 1 differs"
        );
    }

    #[test]
    fn an_unreadable_file_is_counted_only_when_there_is_one() {
        let mut counts = Counts::default();
        counts.record(FileStatus::Unreadable);

        assert_eq!(
            counts.to_string(),
            "0 synced, 0 missing, 0 unlinked, 0 differs, 1 unreadable"
        );
    }

    #[test]
    fn a_template_is_reported_through_the_file_it_produces() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.zshrc.luadot");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("luadot.lua"), r#"return "generated\n""#).unwrap();

        let (produced, counts) = report_templates(
            &Config::default(),
            &home,
            &repo,
            &[Entry::Template(dir)],
            &Classes::default(),
        )
        .unwrap();

        assert_eq!(produced, 1);
        assert_eq!(
            counts.to_string(),
            "0 synced, 1 missing, 0 unlinked, 0 differs"
        );
    }

    #[test]
    fn a_resolved_template_that_is_already_on_the_system_is_synced() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.zshrc.luadot");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(dir.join("luadot.lua"), r#"return "generated\n""#).unwrap();
        std::fs::write(home.join(".zshrc"), "generated\n").unwrap();

        let (produced, counts) = report_templates(
            &Config::default(),
            &home,
            &repo,
            &[Entry::Template(dir)],
            &Classes::default(),
        )
        .unwrap();

        assert_eq!(produced, 1);
        assert_eq!(
            counts.to_string(),
            "1 synced, 0 missing, 0 unlinked, 0 differs"
        );
    }
}
