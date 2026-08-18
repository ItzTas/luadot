use std::path::Path;
use std::sync::atomic::AtomicBool;

use anyhow::{Context, Result};
use gix::progress::Discard;
use tracing::debug;

use super::empty::require_empty;

pub fn clone(dir: &Path, url: &str) -> Result<()> {
    debug!(url, dir = %dir.display(), "cloning");
    require_empty("clone", dir)?;

    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("clone: failed to create {}", parent.display()))?;
    }

    let url = gix::url::parse(url.into()).context("clone: invalid repository url")?;
    let should_interrupt = AtomicBool::new(false);

    let mut fetch = gix::prepare_clone(url, dir).context("clone: failed to prepare clone")?;

    let (mut checkout, _outcome) = fetch
        .fetch_then_checkout(Discard, &should_interrupt)
        .context("clone: failed to fetch repository")?;

    let (_repo, _outcome) = checkout
        .main_worktree(Discard, &should_interrupt)
        .context("clone: failed to checkout worktree")?;

    debug!(dir = %dir.display(), "cloned");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires network access"]
    fn clones_a_public_repo() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("repo");

        clone(&target, "https://github.com/octocat/Hello-World").unwrap();

        assert!(target.join(".git").is_dir());
        assert!(target.join("README").exists());
    }
}
