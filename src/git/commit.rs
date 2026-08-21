use std::path::Path;

use anyhow::Result;
use tracing::debug;

use super::constants::{COMMIT, COMMITTED, MESSAGE, MESSAGE_FLAG, MESSAGE_FROM, STAGED};
use super::run::{present, run, succeeds};
use crate::utils;

pub fn staged(repo: &Path) -> bool {
    present(repo) && !succeeds(repo, STAGED)
}

pub fn committed(repo: &Path) -> bool {
    present(repo) && succeeds(repo, COMMITTED)
}

pub fn commit(command: &str, repo: &Path, message: &str) -> Result<bool> {
    if !staged(repo) {
        return Ok(false);
    }

    debug!(repo = %repo.display(), message, "committing");
    run(command, repo, [COMMIT, MESSAGE_FLAG, message])?;

    Ok(true)
}

pub fn message(custom: Option<String>) -> String {
    custom.unwrap_or_else(|| default(&utils::host_name()))
}

fn default(host: &str) -> String {
    if host.is_empty() {
        return MESSAGE.to_string();
    }

    format!("{MESSAGE_FROM} {host}")
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::super::fixture::{repository, stage};
    use super::*;

    fn subject(repo: &Path) -> String {
        let output = Command::new("git")
            .current_dir(repo)
            .args(["log", "-1", "--format=%s"])
            .output()
            .unwrap();

        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }

    #[test]
    fn an_empty_index_commits_nothing() {
        let repo = repository();

        assert!(!staged(repo.path()));
        assert!(!commit("sync", repo.path(), "unused").unwrap());
        assert!(!committed(repo.path()));
    }

    #[test]
    fn a_staged_file_becomes_a_commit_carrying_the_message() {
        let repo = repository();
        stage(repo.path(), "tracked");

        assert!(staged(repo.path()));
        assert!(commit("sync", repo.path(), "sync from thinkpad").unwrap());

        assert_eq!(subject(repo.path()), "sync from thinkpad");
        assert!(committed(repo.path()));
        assert!(!staged(repo.path()));
    }

    #[test]
    fn the_default_message_names_the_machine() {
        assert_eq!(default("thinkpad"), "sync from thinkpad");
        assert_eq!(default(""), "sync");
    }

    #[test]
    fn a_message_given_on_the_command_line_wins() {
        assert_eq!(message(Some("first".to_string())), "first");
    }
}
