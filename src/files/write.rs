use std::fs::{Metadata, OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use anyhow::{Context, Result, bail};

use super::constants::MODE_BITS;
use super::status::FileStatus;
use super::sync::{ConflictPolicy, SyncOutcome, create_parent, exists, remove_existing};

pub fn text_status(dest: &Path, contents: &str, mode: Option<u32>) -> Result<FileStatus> {
    if !exists(dest)? {
        return Ok(FileStatus::Missing);
    }

    if holds(dest, contents, mode)? {
        return Ok(FileStatus::Synced);
    }

    Ok(FileStatus::Differs)
}

pub fn write_file(
    policy: ConflictPolicy,
    dest: &Path,
    contents: &str,
    mode: Option<u32>,
) -> Result<SyncOutcome> {
    if !exists(dest)? {
        create_parent(dest)?;
        write(dest, contents, mode)?;
        return Ok(SyncOutcome::Created);
    }

    if holds(dest, contents, mode)? {
        return Ok(SyncOutcome::AlreadySynced);
    }

    match policy {
        ConflictPolicy::Skip => return Ok(SyncOutcome::Skipped),
        ConflictPolicy::Error => bail!("files: {} already exists", dest.display()),
        ConflictPolicy::Overwrite => {}
    }

    remove_existing(dest)?;
    write(dest, contents, mode)?;

    Ok(SyncOutcome::Replaced)
}

fn holds(path: &Path, contents: &str, mode: Option<u32>) -> Result<bool> {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return Ok(false);
    };
    if !meta.file_type().is_file() {
        return Ok(false);
    }
    if mode.is_some_and(|mode| bits(&meta) != mode) {
        return Ok(false);
    }

    Ok(matches!(std::fs::read(path), Ok(found) if found == contents.as_bytes()))
}

fn bits(meta: &Metadata) -> u32 {
    meta.permissions().mode() & MODE_BITS
}

fn write(dest: &Path, contents: &str, mode: Option<u32>) -> Result<()> {
    let Some(mode) = mode else {
        return std::fs::write(dest, contents)
            .with_context(|| format!("files: failed to write {}", dest.display()));
    };

    let failed = || format!("files: failed to write {}", dest.display());

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(dest)
        .with_context(failed)?
        .write_all(contents.as_bytes())
        .with_context(failed)?;

    std::fs::set_permissions(dest, Permissions::from_mode(mode))
        .with_context(|| format!("files: failed to set the mode of {}", dest.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_a_missing_destination() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("nested/deep/.zshrc");

        let outcome = write_file(ConflictPolicy::Overwrite, &dest, "generated", None).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "generated");
    }

    #[test]
    fn matching_contents_are_already_synced() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::write(&dest, "generated").unwrap();

        let outcome = write_file(ConflictPolicy::Error, &dest, "generated", None).unwrap();

        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn overwrite_replaces_diverging_contents() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::write(&dest, "stale").unwrap();

        let outcome = write_file(ConflictPolicy::Overwrite, &dest, "generated", None).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "generated");
    }

    #[test]
    fn skip_leaves_the_destination_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::write(&dest, "stale").unwrap();

        let outcome = write_file(ConflictPolicy::Skip, &dest, "generated", None).unwrap();

        assert_eq!(outcome, SyncOutcome::Skipped);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "stale");
    }

    #[test]
    fn error_policy_aborts_on_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::write(&dest, "stale").unwrap();

        assert!(write_file(ConflictPolicy::Error, &dest, "generated", None).is_err());
    }

    #[test]
    fn a_symlink_never_counts_as_the_generated_file() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join(".zshrc");
        std::fs::write(&source, "generated").unwrap();
        std::os::unix::fs::symlink(&source, &dest).unwrap();

        assert!(!holds(&dest, "generated", None).unwrap());

        let outcome = write_file(ConflictPolicy::Overwrite, &dest, "generated", None).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert!(!std::fs::symlink_metadata(&dest).unwrap().is_symlink());
        assert_eq!(std::fs::read_to_string(&source).unwrap(), "generated");
    }

    #[test]
    fn refuses_to_replace_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::create_dir(&dest).unwrap();

        let err = write_file(ConflictPolicy::Overwrite, &dest, "generated", None).unwrap_err();

        assert!(err.to_string().contains("refusing to replace directory"));
    }

    #[test]
    fn a_mode_lands_on_the_file_it_creates() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");

        let outcome = write_file(ConflictPolicy::Overwrite, &dest, "secret", Some(0o600)).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(bits(&std::fs::metadata(&dest).unwrap()), 0o600);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "secret");
    }

    #[test]
    fn a_file_the_umask_widened_is_narrowed_back() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        std::fs::write(&dest, "secret").unwrap();
        std::fs::set_permissions(&dest, Permissions::from_mode(0o644)).unwrap();

        let outcome = write_file(ConflictPolicy::Overwrite, &dest, "secret", Some(0o600)).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(bits(&std::fs::metadata(&dest).unwrap()), 0o600);
    }

    #[test]
    fn a_file_already_carrying_the_mode_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        write_file(ConflictPolicy::Overwrite, &dest, "secret", Some(0o600)).unwrap();

        let outcome = write_file(ConflictPolicy::Overwrite, &dest, "secret", Some(0o600)).unwrap();

        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn a_mode_that_diverges_follows_the_policy() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        write_file(ConflictPolicy::Overwrite, &dest, "secret", Some(0o644)).unwrap();

        let outcome = write_file(ConflictPolicy::Skip, &dest, "secret", Some(0o600)).unwrap();

        assert_eq!(outcome, SyncOutcome::Skipped);
        assert_eq!(bits(&std::fs::metadata(&dest).unwrap()), 0o644);
    }

    #[test]
    fn text_status_reports_a_mode_that_diverges() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        write_file(ConflictPolicy::Overwrite, &dest, "secret", Some(0o644)).unwrap();

        assert_eq!(
            text_status(&dest, "secret", Some(0o600)).unwrap(),
            FileStatus::Differs
        );
        assert_eq!(
            text_status(&dest, "secret", Some(0o644)).unwrap(),
            FileStatus::Synced
        );
        assert_eq!(
            text_status(&dest, "secret", None).unwrap(),
            FileStatus::Synced
        );
    }

    #[test]
    fn text_status_reads_what_the_destination_holds() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");

        assert_eq!(
            text_status(&dest, "generated", None).unwrap(),
            FileStatus::Missing
        );

        std::fs::write(&dest, "generated").unwrap();
        assert_eq!(
            text_status(&dest, "generated", None).unwrap(),
            FileStatus::Synced
        );

        std::fs::write(&dest, "handwritten").unwrap();
        assert_eq!(
            text_status(&dest, "generated", None).unwrap(),
            FileStatus::Differs
        );
    }
}
