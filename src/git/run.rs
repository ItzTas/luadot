use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use super::constants::{GIT_DIR, PROGRAM};

pub fn present(repo: &Path) -> bool {
    repo.join(GIT_DIR).exists()
}

pub(super) fn run<I, S>(command: &str, repo: &Path, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut git = build(repo, args);

    let status = git.status().with_context(|| unavailable(command))?;
    if !status.success() {
        bail!(
            "{command}: `{PROGRAM} {}` exited with status {}",
            head(&git),
            status.code().unwrap_or(1)
        );
    }

    Ok(())
}

pub(super) fn quiet<I, S>(command: &str, repo: &Path, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut git = build(repo, args);

    let output = git.output().with_context(|| unavailable(command))?;
    if !output.status.success() {
        bail!(
            "{command}: `{PROGRAM} {}` failed: {}",
            head(&git),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}

pub(super) fn succeeds<I, S>(repo: &Path, args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    build(repo, args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn build<I, S>(repo: &Path, args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut git = Command::new(PROGRAM);
    git.current_dir(repo);
    git.args(args);
    git
}

fn head(git: &Command) -> String {
    git.get_args()
        .next()
        .map(|arg| arg.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn unavailable(command: &str) -> String {
    format!("{command}: failed to run {PROGRAM}; is it installed and on PATH?")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> tempfile::TempDir {
        let repo = tempfile::tempdir().unwrap();
        assert!(succeeds(repo.path(), ["init", "--quiet"]));

        repo
    }

    #[test]
    fn a_failure_names_the_subcommand() {
        let repo = repository();

        let err = quiet("sync", repo.path(), ["no-such-subcommand"])
            .unwrap_err()
            .to_string();

        assert!(err.contains("sync: `git no-such-subcommand` failed:"));
        assert!(err.contains("no-such-subcommand"));
    }
}
