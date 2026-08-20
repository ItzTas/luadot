use std::path::Path;
use std::sync::atomic::AtomicBool;

use anyhow::{Context, Result};
use tracing::debug;

use super::constants::{CHECKOUT_TASK, FETCH_TASK};
use super::empty::require_empty;
use crate::output::Progress;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cloned {
    Repository,
    Empty,
}

pub fn clone(dir: &Path, url: &str) -> Result<Cloned> {
    debug!(url, dir = %dir.display(), "cloning");
    require_empty("clone", dir)?;

    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("clone: failed to create {}", parent.display()))?;
    }

    let url = gix::url::parse(url.into()).context("clone: invalid repository url")?;
    let should_interrupt = AtomicBool::new(false);

    let mut fetch = gix::prepare_clone(url, dir).context("clone: failed to prepare clone")?;
    let progress = Progress::new();

    let (mut checkout, _outcome) = fetch
        .fetch_then_checkout(progress.task(FETCH_TASK), &should_interrupt)
        .context("clone: failed to fetch repository")?;

    let (repo, _outcome) = checkout
        .main_worktree(progress.task(CHECKOUT_TASK), &should_interrupt)
        .context("clone: failed to checkout worktree")?;

    drop(progress);

    let head = repo.head().context("clone: failed to read HEAD")?;
    let cloned = match head.is_unborn() {
        true => Cloned::Empty,
        false => Cloned::Repository,
    };

    debug!(dir = %dir.display(), ?cloned, "cloned");
    Ok(cloned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires network access"]
    fn clones_a_public_repo() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("repo");

        let cloned = clone(&target, "https://github.com/octocat/Hello-World").unwrap();

        assert_eq!(cloned, Cloned::Repository);
        assert!(target.join(".git").is_dir());
        assert!(target.join("README").exists());
    }
}
