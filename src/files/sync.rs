use std::path::Path;

use anyhow::{Context, Result, bail};
use tracing::debug;

use super::constants::COMMAND;
use super::fs::{create_parent, exists, regular_file, remove_existing};
use super::{LinkMode, link};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictPolicy {
    #[default]
    Overwrite,
    Skip,
    Error,
}

impl ConflictPolicy {
    pub fn name(self) -> &'static str {
        match self {
            Self::Overwrite => "overwrite",
            Self::Skip => "skip",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncOutcome {
    Created,
    Replaced,
    AlreadySynced,
    Skipped,
}

pub fn sync_file(
    policy: ConflictPolicy,
    mode: LinkMode,
    source: &Path,
    dest: &Path,
) -> Result<SyncOutcome> {
    let outcome = sync(policy, mode, source, dest)?;
    debug!(
        source = %source.display(),
        dest = %dest.display(),
        outcome = ?outcome,
        "synced"
    );
    Ok(outcome)
}

fn sync(policy: ConflictPolicy, mode: LinkMode, source: &Path, dest: &Path) -> Result<SyncOutcome> {
    if !source.is_file() {
        bail!("files: {} is not a file", source.display());
    }

    if !exists(COMMAND, dest)? {
        place(mode, source, dest)?;
        return Ok(SyncOutcome::Created);
    }

    if already_synced(mode, source, dest)? {
        return Ok(SyncOutcome::AlreadySynced);
    }

    if let Some(outcome) = refused(COMMAND, policy, dest)? {
        return Ok(outcome);
    }

    remove_existing(COMMAND, dest)?;
    place(mode, source, dest)?;

    Ok(SyncOutcome::Replaced)
}

pub fn refused(command: &str, policy: ConflictPolicy, dest: &Path) -> Result<Option<SyncOutcome>> {
    match policy {
        ConflictPolicy::Skip => Ok(Some(SyncOutcome::Skipped)),
        ConflictPolicy::Error => bail!("{command}: {} already exists", dest.display()),
        ConflictPolicy::Overwrite => Ok(None),
    }
}

fn place(mode: LinkMode, source: &Path, dest: &Path) -> Result<()> {
    create_parent(COMMAND, dest)?;
    match mode {
        LinkMode::Hard => hard_or_copy(source, dest),
        LinkMode::Symbolic | LinkMode::Copy => link(mode, source, dest),
    }
}

fn hard_or_copy(source: &Path, dest: &Path) -> Result<()> {
    match std::fs::hard_link(source, dest) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::CrossesDevices => copy(source, dest),
        Err(err) => Err(err).with_context(|| {
            format!(
                "files: failed to hard link {} -> {}",
                dest.display(),
                source.display()
            )
        }),
    }
}

fn copy(source: &Path, dest: &Path) -> Result<()> {
    std::fs::copy(source, dest).map(|_| ()).with_context(|| {
        format!(
            "files: failed to copy {} -> {}",
            source.display(),
            dest.display()
        )
    })
}

pub(super) fn already_synced(mode: LinkMode, source: &Path, dest: &Path) -> Result<bool> {
    match mode {
        LinkMode::Hard => Ok(same_file(source, dest)),
        LinkMode::Symbolic => points_to(dest, source),
        LinkMode::Copy => copied(source, dest),
    }
}

fn copied(source: &Path, dest: &Path) -> Result<bool> {
    if regular_file(dest).is_none() || same_file(source, dest) {
        return Ok(false);
    }

    same_contents(source, dest)
}

pub(super) fn same_contents(source: &Path, dest: &Path) -> Result<bool> {
    let expected = std::fs::read(source)
        .with_context(|| format!("files: failed to read {}", source.display()))?;

    Ok(matches!(std::fs::read(dest), Ok(found) if found == expected))
}

fn points_to(dest: &Path, source: &Path) -> Result<bool> {
    let Ok(meta) = std::fs::symlink_metadata(dest) else {
        return Ok(false);
    };
    if !meta.file_type().is_symlink() {
        return Ok(false);
    }
    Ok(std::fs::read_link(dest)? == source)
}

fn same_file(source: &Path, dest: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;

    let (Ok(a), Ok(b)) = (std::fs::metadata(source), std::fs::metadata(dest)) else {
        return false;
    };
    a.dev() == b.dev() && a.ino() == b.ino()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::MetadataExt;

    fn write(path: &Path, contents: &str) {
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn default_policy_is_overwrite() {
        assert_eq!(ConflictPolicy::default(), ConflictPolicy::Overwrite);
    }

    #[test]
    fn creates_a_hard_link_when_destination_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
        assert_eq!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );
    }

    #[test]
    fn creates_missing_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("nested/deep/dest");
        write(&source, "data");

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
    }

    #[test]
    fn reports_already_synced_when_hard_linked() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap();

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn overwrite_replaces_a_differing_destination() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "repo");
        write(&dest, "stale");

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "repo");
        assert_eq!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );
    }

    #[test]
    fn skip_leaves_an_existing_destination_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "repo");
        write(&dest, "stale");

        let outcome = sync_file(ConflictPolicy::Skip, LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Skipped);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "stale");
    }

    #[test]
    fn error_policy_aborts_on_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "repo");
        write(&dest, "stale");

        assert!(sync_file(ConflictPolicy::Error, LinkMode::Hard, &source, &dest).is_err());
    }

    #[test]
    fn refuses_to_replace_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "repo");
        std::fs::create_dir(&dest).unwrap();

        let err = sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap_err();
        assert!(err.to_string().contains("refusing to replace directory"));
    }

    #[test]
    fn symbolic_mode_creates_a_symlink_and_detects_it() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");

        let outcome = sync_file(
            ConflictPolicy::Overwrite,
            LinkMode::Symbolic,
            &source,
            &dest,
        )
        .unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert!(
            std::fs::symlink_metadata(&dest)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read_link(&dest).unwrap(), source);

        let outcome = sync_file(
            ConflictPolicy::Overwrite,
            LinkMode::Symbolic,
            &source,
            &dest,
        )
        .unwrap();
        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn copy_mode_creates_an_independent_copy() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Copy, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
        assert_ne!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Copy, &source, &dest).unwrap();
        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn copy_mode_replaces_a_hard_link_with_a_real_copy() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        std::fs::hard_link(&source, &dest).unwrap();

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Copy, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_ne!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );
    }

    #[test]
    fn copy_mode_replaces_a_symlink_with_a_real_copy() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        std::os::unix::fs::symlink(&source, &dest).unwrap();

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Copy, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert!(
            !std::fs::symlink_metadata(&dest)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
    }

    #[test]
    fn errors_when_source_is_not_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("missing");
        let dest = dir.path().join("dest");

        let err = sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap_err();
        assert!(err.to_string().contains("is not a file"));
    }

    #[test]
    fn copy_duplicates_file_contents_into_a_new_inode() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");

        copy(&source, &dest).unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
        assert_ne!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );
    }
}
