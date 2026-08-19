use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;

use crate::crypt;
use crate::files::{self, Entry, FileStatus, Side};
use crate::lua::{Config, StatusCounts, StatusFile};
use crate::output::{self, Tone};
use crate::state::{self, Classes};
use crate::utils::{self, Workspace};

use super::super::constants::{
    CUSTOM_ENTRY, CUSTOM_RENDER, CUSTOM_SUMMARY, STATUS_CLEAN, STATUS_CUSTOM,
    STATUS_GENERATED_CLEAN, STATUS_GENERATED_HEAD, STATUS_HEAD, STATUS_LABELS, STATUS_SECTIONS,
};

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

    let skipped = match args.templates {
        true => 0,
        false => templates.len(),
    };

    output::title(format!("{STATUS_HEAD} {}", repo.display()));
    managed_side(&config, &home, &repo, &files, skipped)?;

    if skipped > 0 || templates.is_empty() {
        return Ok(());
    }

    generated_side(&config, &home, &repo, &templates)
}

fn managed_side(
    config: &Config,
    home: &Path,
    repo: &Path,
    files: &[Entry],
    skipped: usize,
) -> Result<()> {
    if files.is_empty() {
        if skipped > 0 {
            output::line(unresolved(skipped));
        }
        return Ok(());
    }

    let reported = managed_files(config, home, repo, files)?;
    summary(config, Side::Repository, &reported, skipped)?;

    show(config, &reported)
}

fn generated_side(config: &Config, home: &Path, repo: &Path, templates: &[Entry]) -> Result<()> {
    let classes = state::load()?.classes().clone();
    let reported = generated_files(config, home, repo, templates, &classes)?;

    output::section(STATUS_GENERATED_HEAD);
    summary(config, Side::Generated, &reported, templates.len())?;

    show(config, &reported)
}

fn managed_files(
    config: &Config,
    home: &Path,
    repo: &Path,
    files: &[Entry],
) -> Result<Vec<StatusFile>> {
    let lock = config.crypt_lock();
    let mut identity = config.crypt_identity(home);

    let mut reported = Vec::new();
    for file in files.iter().map(Entry::path) {
        let relative = utils::relative(repo, file);
        let split = crypt::split(relative);
        let logical = split
            .as_ref()
            .map(|(stripped, _)| stripped.as_path())
            .unwrap_or(relative);
        let dest = utils::system_path(home, repo, &repo.join(logical))?;
        let status = match (&split, utils::is_root(relative)) {
            (Some((stripped, backend)), true) => crypt::system_status(
                "status",
                *backend,
                lock,
                identity.path("status")?,
                file,
                &dest,
                config.mode(stripped),
            ),
            (Some((_, backend)), false) => crypt::status(
                "status",
                *backend,
                lock,
                identity.path("status")?,
                file,
                &dest,
            ),
            (None, true) => files::inspect_system(file, &dest, config.mode(relative)),
            (None, false) => files::file_status(config.link_mode(relative), file, &dest),
        }
        .with_context(|| format!("status: failed to inspect {}", dest.display()))?;

        reported.push(StatusFile::new(
            relative.to_path_buf(),
            dest,
            Side::Repository,
            status,
        ));
    }

    Ok(reported)
}

fn generated_files(
    config: &Config,
    home: &Path,
    repo: &Path,
    templates: &[Entry],
    classes: &Classes,
) -> Result<Vec<StatusFile>> {
    let mut reported = Vec::new();
    for entry in templates {
        for output in utils::outputs("status", home, repo, entry, classes)? {
            let status = utils::output_status("status", config, home, &output)?;
            let relative = utils::output_relative("status", home, &output)?;

            reported.push(StatusFile::new(
                relative,
                output.dest().to_path_buf(),
                Side::Generated,
                status,
            ));
        }
    }

    Ok(reported)
}

fn show(config: &Config, reported: &[StatusFile]) -> Result<()> {
    if reported.is_empty() {
        return Ok(());
    }

    if let Some(custom) = config.status().render() {
        utils::said(custom.shown(
            &what(CUSTOM_RENDER),
            reported.iter().collect::<Vec<&StatusFile>>(),
        )?);
        return Ok(());
    }

    let Some(custom) = config.status().entry() else {
        grouped(reported);
        return Ok(());
    };

    for file in reported {
        utils::said(custom.shown(&what(CUSTOM_ENTRY), file)?);
    }

    Ok(())
}

fn grouped(reported: &[StatusFile]) {
    for (state, title, hint) in STATUS_SECTIONS {
        let listed: Vec<&StatusFile> = reported
            .iter()
            .filter(|file| file.state() == state)
            .collect();
        if listed.is_empty() {
            continue;
        }

        let (tone, label) = display(state);
        output::section(title);
        output::hint(hint);
        for file in listed {
            output::item(tone, format!("{label}:"), file.path().display());
        }
    }
}

fn summary(config: &Config, side: Side, reported: &[StatusFile], templates: usize) -> Result<()> {
    let counted = counted(reported);
    let default = line(side, reported.len(), templates, &counted);

    let Some(custom) = config.status().summary() else {
        output::line(default);
        return Ok(());
    };

    let counts = StatusCounts::new(side, reported.len(), default)
        .with_templates(templates)
        .with_states(counted.states());

    utils::said(custom.shown(&what(CUSTOM_SUMMARY), &counts)?);

    Ok(())
}

fn line(side: Side, total: usize, templates: usize, counts: &Counts) -> String {
    match side {
        Side::Generated => generated_line(total, templates, counts),
        Side::Repository | Side::System => managed_line(total, templates, counts),
    }
}

fn managed_line(total: usize, skipped: usize, counts: &Counts) -> String {
    let head = match counts.clean() {
        true => STATUS_CLEAN.to_string(),
        false => format!("{total} managed file(s)"),
    };

    match skipped {
        0 => head,
        _ => format!("{head}, {}", unresolved(skipped)),
    }
}

fn generated_line(total: usize, templates: usize, counts: &Counts) -> String {
    if counts.clean() {
        return STATUS_GENERATED_CLEAN.to_string();
    }

    format!("{templates} template(s) into {total} file(s)")
}

fn unresolved(templates: usize) -> String {
    format!("{templates} template(s) not resolved")
}

fn counted(reported: &[StatusFile]) -> Counts {
    let mut counts = Counts::default();
    for file in reported {
        counts.record(file.state());
    }

    counts
}

fn what(key: &str) -> String {
    utils::customized("status", STATUS_CUSTOM, key)
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

    fn states(&self) -> Vec<(FileStatus, u32)> {
        STATUS_LABELS
            .iter()
            .zip(self.0)
            .map(|((kind, _, _), count)| (*kind, count))
            .collect()
    }

    fn clean(&self) -> bool {
        STATUS_LABELS
            .iter()
            .zip(self.0)
            .all(|((kind, _, _), count)| *kind == FileStatus::Synced || count == 0)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::lua::from_source;
    use crate::output::ITEM_WIDTH;

    use super::*;

    fn counts(states: &[FileStatus]) -> Counts {
        let mut counts = Counts::default();
        for status in states {
            counts.record(*status);
        }

        counts
    }

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
    fn the_label_of_a_state_is_the_name_the_configuration_reads() {
        for (status, label, _) in STATUS_LABELS {
            assert_eq!(label, status.name());
        }
    }

    #[test]
    fn labels_fit_the_printed_column() {
        for (_, text, _) in STATUS_LABELS {
            assert!(text.len() + 1 < ITEM_WIDTH);
        }
    }

    #[test]
    fn every_state_a_file_can_be_reported_in_has_a_section() {
        for (status, _, _) in STATUS_LABELS {
            if status == FileStatus::Synced {
                continue;
            }

            assert!(STATUS_SECTIONS.iter().any(|(state, _, _)| *state == status));
        }
    }

    #[test]
    fn the_states_are_counted_one_by_one_for_the_configuration() {
        let states = counts(&[FileStatus::Synced, FileStatus::Differs]).states();

        assert_eq!(states.len(), STATUS_LABELS.len());
        assert!(states.contains(&(FileStatus::Synced, 1)));
        assert!(states.contains(&(FileStatus::Differs, 1)));
        assert!(states.contains(&(FileStatus::Missing, 0)));
    }

    #[test]
    fn each_side_says_what_it_reports() {
        let counts = counts(&[FileStatus::Synced, FileStatus::Missing]);

        assert_eq!(line(Side::Repository, 2, 0, &counts), "2 managed file(s)");
        assert_eq!(
            line(Side::Generated, 2, 1, &counts),
            "1 template(s) into 2 file(s)"
        );
    }

    #[test]
    fn a_side_with_nothing_to_apply_says_so() {
        let counts = counts(&[FileStatus::Synced, FileStatus::Synced]);

        assert_eq!(line(Side::Repository, 2, 0, &counts), STATUS_CLEAN);
        assert_eq!(line(Side::Generated, 2, 1, &counts), STATUS_GENERATED_CLEAN);
    }

    #[test]
    fn the_templates_left_alone_close_the_managed_line() {
        let counts = counts(&[FileStatus::Synced]);

        assert_eq!(
            line(Side::Repository, 1, 2, &counts),
            "nothing to apply, every managed file is synced, 2 template(s) not resolved"
        );
    }

    #[test]
    fn a_customized_entry_reads_the_state_of_the_file() {
        let config = from_source(
            r#"ld.on.status({ entry = function(file)
                 return file.side .. " " .. file.state .. " " .. file.path
               end })"#,
        )
        .unwrap();

        let file = StatusFile::new(
            PathBuf::from("home/.bashrc"),
            PathBuf::from("/home/u/.bashrc"),
            Side::Repository,
            FileStatus::Unlinked,
        );

        let shown = config
            .status()
            .entry()
            .unwrap()
            .shown(&what(CUSTOM_ENTRY), &file)
            .unwrap();

        assert_eq!(shown, Some("repository unlinked home/.bashrc".to_string()));
    }

    #[test]
    fn a_customized_summary_reads_the_counts_of_the_side() {
        let config = from_source(
            r#"ld.on.status({ summary = function(counts)
                 return counts.templates .. " into " .. counts.total .. ", " .. counts.synced
               end })"#,
        )
        .unwrap();

        let counts = StatusCounts::new(Side::Generated, 3, "unused".to_string())
            .with_templates(2)
            .with_states(counts(&[FileStatus::Synced, FileStatus::Synced]).states());

        let shown = config
            .status()
            .summary()
            .unwrap()
            .shown(&what(CUSTOM_SUMMARY), &counts)
            .unwrap();

        assert_eq!(shown, Some("2 into 3, 2".to_string()));
    }

    #[test]
    fn a_template_is_reported_through_the_file_it_produces() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join("home/.zshrc.luadot");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("luadot.lua"), r#"return "generated\n""#).unwrap();

        let reported = generated_files(
            &Config::default(),
            &home,
            &repo,
            &[Entry::Template(dir)],
            &Classes::default(),
        )
        .unwrap();

        assert_eq!(reported.len(), 1);
        assert_eq!(reported[0].state(), FileStatus::Missing);
        assert_eq!(reported[0].path(), Path::new("home/.zshrc"));
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

        let reported = generated_files(
            &Config::default(),
            &home,
            &repo,
            &[Entry::Template(dir)],
            &Classes::default(),
        )
        .unwrap();

        assert_eq!(reported.len(), 1);
        assert_eq!(reported[0].state(), FileStatus::Synced);
    }
}
