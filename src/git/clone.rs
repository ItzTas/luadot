use std::path::Path;
use std::sync::atomic::AtomicBool;

use anyhow::{Context, Result};
use gix::sec::Trust;
use gix::sec::trust::DefaultForLevel;
use tracing::debug;

use super::constants::{CHECKOUT_TASK, FETCH_TASK};
use super::empty::require_empty;
use super::{info, lfs};
use crate::output::Progress;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cloned {
    Repository,
    Empty,
}

pub fn clone(dir: &Path, url: &str, lfs: bool) -> Result<Cloned> {
    debug!(url, dir = %dir.display(), lfs, "cloning");
    require_empty("clone", dir)?;

    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("clone: failed to create {}", parent.display()))?;
    }

    let url = gix::url::parse(url.into()).context("clone: invalid repository url")?;
    let should_interrupt = AtomicBool::new(false);

    let mut fetch = gix::clone::PrepareFetch::new(
        url,
        dir,
        gix::create::Kind::WithWorktree,
        gix::create::Options::default(),
        options(lfs),
    )
    .context("clone: failed to prepare clone")?;
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

    lfs::install("clone", dir, lfs)?;
    if info::refresh("clone", dir)? {
        lfs::pull("clone", dir, lfs)?;
    }

    debug!(dir = %dir.display(), ?cloned, "cloned");
    Ok(cloned)
}

fn options(lfs: bool) -> gix::open::Options {
    let mut options = gix::open::Options::default_for_level(Trust::Full);
    options.permissions.config.git_binary = true;

    options.config_overrides(lfs::filters(lfs).iter().copied())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::git::fixture::repository;

    const CONTENTS: &str = "a large file\n";

    fn origin() -> tempfile::TempDir {
        let repo = repository();
        let rules = super::super::rules::dir("test", repo.path()).unwrap();
        std::fs::create_dir_all(&rules).unwrap();
        std::fs::write(
            rules.join(super::super::constants::RULES_ATTRIBUTES),
            "*.bin filter=lfs diff=lfs merge=lfs -text\n",
        )
        .unwrap();
        assert!(info::refresh("test", repo.path()).unwrap());
        std::fs::write(repo.path().join("big.bin"), CONTENTS).unwrap();

        for args in [
            vec!["lfs", "install", "--local"],
            vec!["add", "-A"],
            vec!["commit", "--quiet", "-m", "init"],
        ] {
            super::super::run::quiet("test", repo.path(), args).unwrap();
        }

        repo
    }

    #[test]
    fn cloning_pulls_a_file_the_attributes_of_the_repository_send_through_lfs() {
        if !lfs::available() {
            return;
        }
        let origin = origin();
        let dir = tempfile::tempdir().unwrap();
        let into = dir.path().join("clone");

        let cloned = clone(&into, &origin.path().to_string_lossy(), true).unwrap();

        assert_eq!(cloned, Cloned::Repository);
        assert_eq!(
            std::fs::read_to_string(into.join("big.bin")).unwrap(),
            CONTENTS
        );
        assert!(
            std::fs::read_to_string(into.join(".git/info/attributes"))
                .unwrap()
                .contains("*.bin filter=lfs")
        );
        assert!(
            std::fs::read_to_string(into.join(".git/config"))
                .unwrap()
                .contains("filter-process")
        );
    }
}
