use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::files::{self, Entry, FileStatus, Mirror, Side};
use crate::lua::Config;
use crate::output::{self, Tone};
use crate::utils::{self, Workspace};

use super::super::constants::DIFF_ARGUMENTS;

#[derive(Debug, Args)]
pub struct DiffArgs {
    #[arg(value_name = "PATH")]
    pub path: Option<String>,
}

enum System {
    Absent,
    Other,
    Holds(Vec<u8>, u32),
}

pub fn diff_cmd(args: DiffArgs) -> Result<()> {
    let Workspace { config, home, repo } = utils::workspace("diff")?;

    let root = utils::managed_root("diff", &home, &repo, args.path.as_deref())?;

    let entries =
        utils::managed_entries("diff", &repo, &root, |relative| config.is_ignored(relative))?;

    let templates = entries
        .iter()
        .filter(|entry| !matches!(entry, Entry::File(_)))
        .count();
    let files: Vec<PathBuf> = entries
        .into_iter()
        .filter_map(|entry| match entry {
            Entry::File(file) => Some(file),
            Entry::Template(_) | Entry::Standalone(_) => None,
        })
        .collect();

    if files.is_empty() && templates == 0 {
        output::note("nothing is managed");
        return Ok(());
    }

    let drifted = drifted(&config, &home, &repo, &files)?;
    if !drifted.is_empty() {
        show(&config, &repo, &drifted)?;
    }

    output::note(summary(drifted.len(), files.len(), templates));

    Ok(())
}

fn drifted(
    config: &Config,
    home: &Path,
    repo: &Path,
    files: &[PathBuf],
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut drifted = Vec::new();
    for file in files {
        let relative = utils::relative(repo, file);
        let dest = utils::system_path(home, repo, file)?;
        let status = match utils::is_root(relative) {
            true => files::escalated_status(file, &dest, config.mode(relative)),
            false => files::file_status(config.link_mode(relative), file, &dest),
        }
        .with_context(|| format!("diff: failed to inspect {}", dest.display()))?;

        if !shows(status) {
            continue;
        }
        drifted.push((file.clone(), dest));
    }

    Ok(drifted)
}

fn show(config: &Config, repo: &Path, drifted: &[(PathBuf, PathBuf)]) -> Result<()> {
    let mirror = Mirror::open("diff")?;

    let mut staged = 0u32;
    for (file, dest) in drifted {
        let relative = utils::relative(repo, file);
        let expected = files::effective_mode(file, config.mode(relative))?;
        let contents = files::read_contents("diff", file)?;

        match system_side(dest)? {
            System::Other => output::entry(Tone::Warning, "not a file", relative.display()),
            System::Absent => {
                mirror.place(Side::Repository, relative, &contents, expected)?;
                staged += 1;
            }
            System::Holds(found, mode) if found == contents => {
                output::entry(
                    Tone::Warning,
                    "mode",
                    format!("{} {mode:04o} -> {expected:04o}", relative.display()),
                );
            }
            System::Holds(found, mode) => {
                mirror.place(Side::Repository, relative, &contents, expected)?;
                mirror.place(Side::System, relative, &found, mode)?;
                staged += 1;
            }
        }
    }

    if staged == 0 {
        return Ok(());
    }

    run(mirror.root())
}

fn system_side(dest: &Path) -> Result<System> {
    let Ok(meta) = std::fs::metadata(dest) else {
        return Ok(System::Absent);
    };
    if !meta.is_file() {
        return Ok(System::Other);
    }

    let mode = files::effective_mode(dest, None)?;

    Ok(System::Holds(files::read_contents("diff", dest)?, mode))
}

fn run(root: &Path) -> Result<()> {
    let status = build_command(root)
        .status()
        .context("diff: failed to run git; is it installed and on PATH?")?;

    match status.code() {
        None | Some(0 | 1) => Ok(()),
        Some(code) => bail!("diff: git exited with status {code}"),
    }
}

fn build_command(root: &Path) -> Command {
    let mut command = Command::new("git");
    command.current_dir(root);
    command.args(DIFF_ARGUMENTS);
    command.args([Side::Repository.dir(), Side::System.dir()]);
    command
}

fn shows(status: FileStatus) -> bool {
    matches!(
        status,
        FileStatus::Missing | FileStatus::Differs | FileStatus::Unreadable
    )
}

fn summary(shown: usize, total: usize, templates: usize) -> String {
    let mut summary = format!("{shown} of {total} managed file(s) differ");
    if templates > 0 {
        summary.push_str(&format!(", {templates} template(s) skipped"));
    }

    summary
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn only_the_states_with_something_to_show_are_diffed() {
        assert!(shows(FileStatus::Missing));
        assert!(shows(FileStatus::Differs));
        assert!(shows(FileStatus::Unreadable));
        assert!(!shows(FileStatus::Synced));
        assert!(!shows(FileStatus::Unlinked));
    }

    #[test]
    fn the_summary_counts_what_was_shown() {
        assert_eq!(summary(2, 14, 0), "2 of 14 managed file(s) differ");
        assert_eq!(
            summary(0, 3, 1),
            "0 of 3 managed file(s) differ, 1 template(s) skipped"
        );
    }

    #[test]
    fn git_compares_the_two_sides_from_inside_the_mirror() {
        let command = build_command(Path::new("/tmp/luadot-diff-1-0"));

        assert_eq!(command.get_program(), OsStr::new("git"));
        assert_eq!(
            command.get_current_dir(),
            Some(Path::new("/tmp/luadot-diff-1-0"))
        );

        let args: Vec<&str> = command.get_args().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(
            args,
            [
                "diff",
                "--no-index",
                "--no-prefix",
                "--",
                "repository",
                "system"
            ]
        );
    }

    #[test]
    fn the_system_side_is_read_only_when_it_is_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(".bashrc");
        std::fs::write(&file, "handwritten\n").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();

        let System::Holds(contents, mode) = system_side(&file).unwrap() else {
            panic!("a regular file holds its contents");
        };
        assert_eq!(contents, b"handwritten\n");
        assert_eq!(mode, 0o600);

        assert!(matches!(
            system_side(&dir.path().join("gone")).unwrap(),
            System::Absent
        ));
        assert!(matches!(system_side(dir.path()).unwrap(), System::Other));

        let dangling = dir.path().join(".zshrc");
        std::os::unix::fs::symlink(dir.path().join("gone"), &dangling).unwrap();
        assert!(matches!(system_side(&dangling).unwrap(), System::Absent));
    }
}
