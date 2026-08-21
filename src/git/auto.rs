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

    use super::super::fixture::{repository, stage};
    use super::*;

    fn commits(repo: &Path) -> String {
        let output = Command::new("git")
            .current_dir(repo)
            .args(["log", "--format=%s"])
            .output()
            .unwrap();

        String::from_utf8(output.stdout).unwrap()
    }

    #[test]
    fn the_staged_files_become_a_commit() {
        let repo = repository();
        stage(repo.path(), "tracked");

        auto("add", repo.path(), true, false).unwrap();

        assert_eq!(commits(repo.path()).lines().count(), 1);
    }
}
