use std::num::NonZeroU32;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use anyhow::{Context, Result};
use gix::clone::PrepareFetch;
use gix::remote::fetch::Shallow;
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
    let fetch = prepare("clone", dir, url, options(lfs))?;
    let repo = checkout("clone", fetch)?;

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

pub fn clone_plain(
    command: &str,
    dir: &Path,
    url: &str,
    branch: Option<&str>,
    depth: Option<NonZeroU32>,
) -> Result<()> {
    debug!(url, dir = %dir.display(), branch, depth, "cloning");
    let mut fetch = prepare(command, dir, url, options(false))?
        .with_ref_name(branch)
        .with_context(|| format!("{command}: invalid branch name"))?;
    if let Some(depth) = depth {
        fetch = fetch.with_shallow(Shallow::DepthAtRemote(depth));
    }

    checkout(command, fetch)?;

    debug!(dir = %dir.display(), "cloned");
    Ok(())
}

fn prepare(
    command: &str,
    dir: &Path,
    url: &str,
    options: gix::open::Options,
) -> Result<PrepareFetch> {
    require_empty(command, dir)?;

    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("{command}: failed to create {}", parent.display()))?;
    }

    let url = gix::url::parse(url.into())
        .with_context(|| format!("{command}: invalid repository url"))?;

    PrepareFetch::new(
        url,
        dir,
        gix::create::Kind::WithWorktree,
        gix::create::Options::default(),
        options,
    )
    .with_context(|| format!("{command}: failed to prepare clone"))
}

fn checkout(command: &str, mut fetch: PrepareFetch) -> Result<gix::Repository> {
    let should_interrupt = AtomicBool::new(false);
    let progress = Progress::new();

    let (mut checkout, _outcome) = fetch
        .fetch_then_checkout(progress.task(FETCH_TASK), &should_interrupt)
        .with_context(|| format!("{command}: failed to fetch repository"))?;

    let (repo, _outcome) = checkout
        .main_worktree(progress.task(CHECKOUT_TASK), &should_interrupt)
        .with_context(|| format!("{command}: failed to checkout worktree"))?;

    drop(progress);

    Ok(repo)
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

    fn commit(repo: &Path, name: &str) {
        std::fs::write(repo.join(name), name).unwrap();
        for args in [vec!["add", "-A"], vec!["commit", "--quiet", "-m", name]] {
            super::super::run::quiet("test", repo, args).unwrap();
        }
    }

    fn commits(repo: &Path) -> String {
        let output = std::process::Command::new("git")
            .current_dir(repo)
            .args(["rev-list", "--count", "HEAD"])
            .output()
            .unwrap();

        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }

    #[test]
    fn cloning_pulls_an_lfs_file() {
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

    #[test]
    fn a_clone_takes_branch_and_depth() {
        let origin = repository();
        commit(origin.path(), "first");
        super::super::run::quiet(
            "test",
            origin.path(),
            ["checkout", "--quiet", "-b", "feature"],
        )
        .unwrap();
        commit(origin.path(), "second");
        let dir = tempfile::tempdir().unwrap();
        let into = dir.path().join("plugins/feature");

        clone_plain(
            "test",
            &into,
            &origin.path().to_string_lossy(),
            Some("feature"),
            NonZeroU32::new(1),
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(into.join("second")).unwrap(),
            "second"
        );
        assert_eq!(commits(&into), "1");
        assert!(!into.join(".git/info/attributes").exists());
    }

    #[test]
    fn a_clone_refuses_a_full_directory() {
        let origin = repository();
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("kept"), "").unwrap();

        let err = clone_plain(
            "test",
            dir.path(),
            &origin.path().to_string_lossy(),
            None,
            None,
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("test: destination"));
        assert!(err.contains("is not empty"));
    }
}
