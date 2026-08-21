use std::path::Path;

use anyhow::{Result, bail};
use tracing::debug;

use super::commit::committed;
use super::constants::{PUSH, SET_UPSTREAM, UPSTREAM};
use super::run::{run, succeeds};

pub fn push(command: &str, repo: &Path) -> Result<()> {
    if !committed(repo) {
        bail!("{command}: nothing is committed yet, so there is nothing to push");
    }

    debug!(repo = %repo.display(), "pushing");
    run(command, repo, arguments(tracked(repo)))
}

fn tracked(repo: &Path) -> bool {
    succeeds(repo, UPSTREAM)
}

fn arguments(tracked: bool) -> Vec<&'static str> {
    if tracked {
        return vec![PUSH];
    }

    std::iter::once(PUSH).chain(SET_UPSTREAM).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repository_without_a_commit_is_refused() {
        let dir = tempfile::tempdir().unwrap();

        let err = push("sync", dir.path()).unwrap_err().to_string();

        assert!(err.contains("sync: nothing is committed yet"));
    }
}
