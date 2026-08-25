use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::crypt;
use crate::files::{self, Entry, FileStatus, Side};
use crate::git;
use crate::lua::{Command, Config, Shared, StatusCounts, StatusFile};
use crate::output::{self, Tone};
use crate::state::{self, Classes};
use crate::utils::{self, Workspace};

use super::super::constants::{
    CUSTOM_ENTRY, CUSTOM_RENDER, CUSTOM_SUMMARY, STATUS_CLEAN, STATUS_GENERATED_CLEAN,
    STATUS_GENERATED_HEAD, STATUS_HEAD, STATUS_LABELS, STATUS_SECTIONS,
};

#[derive(Debug, Args)]
pub struct StatusArgs {
    #[arg(
        value_name = "PATH",
        help = "Narrow the report to this file or directory"
    )]
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
    let Workspace {
        config: shared,
        home,
        repo,
    } = utils::workspace("status")?;
    let config = utils::configured("status", &shared)?;

    let root = utils::managed_root("status", &home, &repo, args.path.as_deref())?;

    let (files, templates): (Vec<Entry>, Vec<Entry>) =
        utils::managed_entries("status", &repo, &root, |relative| {
            config.is_ignored(&crypt::logical(relative))
        })?
        .into_iter()
        .partition(|entry| matches!(entry, Entry::File(_)));

    if files.is_empty() && templates.is_empty() {
        output::note("nothing is managed");
        return waiting(&config, &home, &repo);
    }

    let skipped = match args.templates {
        true => 0,
        false => templates.len(),
    };

    output::title(format!("{STATUS_HEAD} {}", repo.display()));
    managed_side(&config, &home, &repo, &files, skipped)?;
    waiting(&config, &home, &repo)?;

    if skipped > 0 || templates.is_empty() {
        return Ok(());
    }
    drop(config);

    generated_side(&shared, &home, &repo, &templates)
}

fn waiting(config: &Config, home: &Path, repo: &Path) -> Result<()> {
    if config.adoption_roots().is_empty() {
        return Ok(());
    }

    let mut excludes = git::Excludes::open("status", repo)?;
    let waiting = utils::adoptable("status", home, repo, config, &mut excludes)?.len();
    if waiting == 0 {
        return Ok(());
    }

    output::note(format!(
        "{waiting} file(s) an `auto` rule covers, not managed yet; `luadot add` takes them"
    ));

    Ok(())
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

fn generated_side(shared: &Shared, home: &Path, repo: &Path, templates: &[Entry]) -> Result<()> {
    let classes = state::load()?.classes().clone();
    let reported = generated_files(shared, home, repo, templates, &classes)?;
    let config = utils::configured("status", shared)?;

    output::section(STATUS_GENERATED_HEAD);
    summary(&config, Side::Generated, &reported, templates.len())?;

    show(&config, &reported)
}

fn managed_files(
    config: &Config,
    home: &Path,
    repo: &Path,
    files: &[Entry],
) -> Result<Vec<StatusFile>> {
    let lock = config.crypt_lock();
    let mut identity = config.crypt_identity(home);

    let paths: Vec<PathBuf> = files
        .iter()
        .map(|entry| entry.path().to_path_buf())
        .collect();
    let managed = utils::units("status", config, repo, paths)?;

    let mut reported = Vec::new();
    for one in &managed {
        reported.push(match one {
            utils::Managed::Unit(unit) => unit_file(config, home, repo, unit)?,
            utils::Managed::File(file) => {
                status_file(config, lock, &mut identity, home, repo, file)?
            }
        });
    }

    Ok(reported)
}

fn unit_file(config: &Config, home: &Path, repo: &Path, unit: &utils::Unit) -> Result<StatusFile> {
    let relative = utils::relative(repo, unit.root());
    let dest = utils::system_path(home, repo, unit.root())?;
    let link = utils::whole_link("status", config, relative)?;
    let status = files::dir_status(link, unit.root(), &dest)
        .with_context(|| format!("status: failed to inspect {}", dest.display()))?;

    Ok(StatusFile::new(
        relative.to_path_buf(),
        dest,
        Side::Repository,
        status,
    ))
}

fn status_file(
    config: &Config,
    lock: crypt::Lock,
    identity: &mut crypt::Identity,
    home: &Path,
    repo: &Path,
    file: &Path,
) -> Result<StatusFile> {
    let relative = utils::relative(repo, file);
    let split = crypt::split(relative);
    let logical = split
        .as_ref()
        .map(|(stripped, _)| stripped.as_path())
        .unwrap_or(relative);
    let dest = utils::system_path(home, repo, &repo.join(logical))?;
    let status = match &split {
        Some((stripped, backend)) => crypt::status(
            "status",
            *backend,
            lock,
            identity.path("status")?,
            file,
            &dest,
            config.mode(stripped),
        ),
        None => files::file_status(config.placement(relative), file, &dest),
    }
    .with_context(|| format!("status: failed to inspect {}", dest.display()))?;

    Ok(StatusFile::new(
        logical.to_path_buf(),
        dest,
        Side::Repository,
        status,
    ))
}

fn generated_files(
    shared: &Shared,
    home: &Path,
    repo: &Path,
    templates: &[Entry],
    classes: &Classes,
) -> Result<Vec<StatusFile>> {
    let mut reported = Vec::new();
    for entry in templates {
        for output in utils::outputs("status", home, repo, entry, classes, shared)? {
            let status = {
                let config = utils::configured("status", shared)?;
                utils::output_status("status", &config, home, &output)?
            };
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
    for (state, title, hints) in STATUS_SECTIONS {
        let listed: Vec<&StatusFile> = reported
            .iter()
            .filter(|file| file.state() == state)
            .collect();
        if listed.is_empty() {
            continue;
        }

        let (tone, label) = display(state);
        output::section(title);
        for line in hints {
            output::hint(line);
        }
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
    utils::customized("status", &Command::Status.call(), key)
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
    use std::sync::{Arc, Mutex};

    use super::*;

    fn counts(states: &[FileStatus]) -> Counts {
        let mut counts = Counts::default();
        for status in states {
            counts.record(*status);
        }

        counts
    }

    #[test]
    fn every_state_is_counted() {
        let states = counts(&[FileStatus::Synced, FileStatus::Differs]).states();

        assert_eq!(states.len(), STATUS_LABELS.len());
        assert!(states.contains(&(FileStatus::Synced, 1)));
        assert!(states.contains(&(FileStatus::Differs, 1)));
        assert!(states.contains(&(FileStatus::Missing, 0)));
    }

    #[test]
    fn a_template_reports_its_output() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let dir = repo.join(".zshrc.luadot");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("luadot.lua"), r#"return "generated\n""#).unwrap();

        let reported = generated_files(
            &Arc::new(Mutex::new(Config::default())),
            &home,
            &repo,
            &[Entry::Template(dir)],
            &Classes::default(),
        )
        .unwrap();

        assert_eq!(reported.len(), 1);
        assert_eq!(reported[0].state(), FileStatus::Missing);
        assert_eq!(reported[0].path(), Path::new(".zshrc"));
    }
}
