use std::path::Path;
use std::sync::atomic::AtomicBool;

use anyhow::{Context, Result, bail};
use gix::progress::Discard;

pub fn clone(dir: &Path, url: &str) -> Result<()> {
    if dir.exists() {
        let mut entries = std::fs::read_dir(dir)
            .with_context(|| format!("clone: failed to read {}", dir.display()))?;
        if entries.next().is_some() {
            bail!("clone: destination {} is not empty", dir.display());
        }
    }

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

    #[test]
    fn fails_when_destination_is_not_empty() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("repo");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("existing.txt"), "data").unwrap();

        let err = clone(&target, "https://example.com/repo.git").unwrap_err();

        assert!(err.to_string().contains("not empty"));
    }
}
