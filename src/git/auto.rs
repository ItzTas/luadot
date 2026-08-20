use std::path::Path;

use anyhow::Result;

use super::commit::{commit, committed, message};
use super::push::push;
use super::run::present;

pub fn auto(command: &str, repo: &Path, commits: bool, pushes: bool) -> Result<()> {
    if !present(repo) || (!commits && !pushes) {
        return Ok(());
    }

    if commits {
        commit(command, repo, &message(None))?;
    }
    if !pushes || !committed(repo) {
        return Ok(());
    }

    push(command, repo)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    fn repository() -> tempfile::TempDir {
        let repo = tempfile::tempdir().unwrap();

        for args in [
            vec!["init", "--quiet"],
            vec!["config", "user.email", "test@luadot"],
            vec!["config", "user.name", "luadot"],
            vec!["config", "commit.gpgsign", "false"],
        ] {
            let status = Command::new("git")
                .current_dir(repo.path())
                .args(args)
                .status()
                .unwrap();
            assert!(status.success());
        }

        repo
    }

    fn stage(repo: &Path) {
        std::fs::write(repo.join("tracked"), "contents\n").unwrap();
        let status = Command::new("git")
            .current_dir(repo)
            .args(["add", "tracked"])
            .status()
            .unwrap();
        assert!(status.success());
    }

    fn commits(repo: &Path) -> String {
        let output = Command::new("git")
            .current_dir(repo)
            .args(["log", "--format=%s"])
            .output()
            .unwrap();

        String::from_utf8(output.stdout).unwrap()
    }

    #[test]
    fn an_index_asking_for_no_commit_is_left_staged() {
        let repo = repository();
        stage(repo.path());

        auto("add", repo.path(), false, false).unwrap();

        assert!(commits(repo.path()).is_empty());
    }

    #[test]
    fn the_staged_files_become_a_commit() {
        let repo = repository();
        stage(repo.path());

        auto("add", repo.path(), true, false).unwrap();

        assert_eq!(commits(repo.path()).lines().count(), 1);
    }

    #[test]
    fn a_repository_without_a_commit_pushes_nothing() {
        let repo = repository();

        auto("add", repo.path(), true, true).unwrap();

        assert!(commits(repo.path()).is_empty());
    }
}
