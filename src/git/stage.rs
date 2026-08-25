use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::debug;

use super::constants::{ADD, ADD_ALL, REMOVE, SEPARATOR};
use super::run::{present, quiet};

pub fn stage(command: &str, repo: &Path, paths: &[PathBuf]) -> Result<()> {
    change(command, repo, &ADD, paths)
}

pub fn unstage(command: &str, repo: &Path, paths: &[PathBuf]) -> Result<()> {
    change(command, repo, &REMOVE, paths)
}

pub fn stage_all(command: &str, repo: &Path) -> Result<()> {
    if !present(repo) {
        return Ok(());
    }

    debug!(repo = %repo.display(), "staging everything");
    quiet(command, repo, ADD_ALL)
}

fn change(command: &str, repo: &Path, head: &[&str], paths: &[PathBuf]) -> Result<()> {
    if paths.is_empty() || !present(repo) {
        return Ok(());
    }

    debug!(repo = %repo.display(), paths = paths.len(), "staging");
    quiet(command, repo, arguments(head, paths))
}

fn arguments<'a>(head: &'a [&'a str], paths: &'a [PathBuf]) -> Vec<&'a OsStr> {
    let mut arguments: Vec<&OsStr> = head.iter().copied().map(OsStr::new).collect();
    arguments.push(OsStr::new(SEPARATOR));
    arguments.extend(paths.iter().map(|path| path.as_os_str()));

    arguments
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    fn repository() -> tempfile::TempDir {
        let repo = tempfile::tempdir().unwrap();
        let status = Command::new("git")
            .current_dir(repo.path())
            .args(["init", "--quiet"])
            .status()
            .unwrap();
        assert!(status.success());

        repo
    }

    fn index(repo: &Path) -> String {
        let output = Command::new("git")
            .current_dir(repo)
            .args(["diff", "--cached", "--name-only"])
            .output()
            .unwrap();

        String::from_utf8(output.stdout).unwrap()
    }

    #[test]
    fn staging_puts_the_paths_in_the_index() {
        let repo = repository();
        let file = repo.path().join(".bashrc");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "alias l=ls\n").unwrap();

        stage("add", repo.path(), &[file]).unwrap();

        assert_eq!(index(repo.path()), ".bashrc\n");
    }

    #[test]
    fn unstaging_records_the_removal_of_a_path_already_gone() {
        let repo = repository();
        let file = repo.path().join(".bashrc");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "alias l=ls\n").unwrap();
        stage("add", repo.path(), std::slice::from_ref(&file)).unwrap();
        std::fs::remove_file(&file).unwrap();

        unstage("rm", repo.path(), &[file]).unwrap();

        assert_eq!(index(repo.path()), "");
    }
}
