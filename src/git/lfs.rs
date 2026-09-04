use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::Result;
use tracing::debug;

use super::constants::LFS_FILTERS;
use super::run::{present, quiet};

pub fn available() -> bool {
    Command::new("git-lfs")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

pub fn wanted(lfs: bool) -> bool {
    lfs && available()
}

pub fn filters(lfs: bool) -> &'static [&'static str] {
    match wanted(lfs) {
        true => &LFS_FILTERS,
        false => &[],
    }
}

pub fn install(command: &str, repo: &Path, lfs: bool) -> Result<()> {
    if !wanted(lfs) || !present(repo) {
        return Ok(());
    }

    debug!(repo = %repo.display(), "installing the lfs filters");
    quiet(command, repo, ["lfs", "install", "--local"])
}

pub fn pull(command: &str, repo: &Path, lfs: bool) -> Result<()> {
    if !wanted(lfs) || !present(repo) {
        return Ok(());
    }

    debug!(repo = %repo.display(), "pulling the lfs contents");
    quiet(command, repo, ["lfs", "pull"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::fixture::repository;

    fn configured(repo: &Path) -> String {
        let output = Command::new("git")
            .current_dir(repo)
            .args(["config", "--local", "--get-regexp", "^filter\\.lfs"])
            .output()
            .unwrap();

        String::from_utf8(output.stdout).unwrap()
    }

    #[test]
    fn installing_writes_the_filters() {
        let repo = repository();

        install("clone", repo.path(), true).unwrap();

        if !available() {
            assert_eq!(configured(repo.path()), "");
            return;
        }
        assert!(configured(repo.path()).contains("filter.lfs.process"));
    }
}
