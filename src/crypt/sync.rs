use std::path::Path;

use anyhow::Result;

use super::backend::Backend;
use super::constants::SECRET_MODE;
use super::run;
use crate::files::{
    ConflictPolicy, FileStatus, SyncOutcome, create_parent, exists, mode_bits, refused,
    regular_file, remove_existing, write_mode,
};

pub fn status(
    command: &str,
    backend: Backend,
    identity: Option<&Path>,
    source: &Path,
    dest: &Path,
) -> Result<FileStatus> {
    if !exists(command, dest)? {
        return Ok(FileStatus::Missing);
    }

    let Ok(expected) = run::decrypt(command, backend, identity, source) else {
        return Ok(FileStatus::Unreadable);
    };

    plain_status(command, &expected, dest)
}

pub fn plain_status(command: &str, expected: &[u8], dest: &Path) -> Result<FileStatus> {
    if !exists(command, dest)? {
        return Ok(FileStatus::Missing);
    }

    if holds(dest, expected) {
        return Ok(FileStatus::Synced);
    }

    Ok(FileStatus::Differs)
}

pub fn place(
    command: &str,
    policy: ConflictPolicy,
    contents: &[u8],
    dest: &Path,
) -> Result<SyncOutcome> {
    if !exists(command, dest)? {
        create_parent(command, dest)?;
        write(command, dest, contents)?;
        return Ok(SyncOutcome::Created);
    }

    if holds(dest, contents) {
        return Ok(SyncOutcome::AlreadySynced);
    }

    if let Some(outcome) = refused(command, policy, dest)? {
        return Ok(outcome);
    }

    remove_existing(command, dest)?;
    write(command, dest, contents)?;

    Ok(SyncOutcome::Replaced)
}

fn holds(dest: &Path, expected: &[u8]) -> bool {
    let Some(meta) = regular_file(dest) else {
        return false;
    };
    if mode_bits(&meta) != SECRET_MODE {
        return false;
    }

    matches!(std::fs::read(dest), Ok(found) if found == expected)
}

fn write(command: &str, dest: &Path, contents: &[u8]) -> Result<()> {
    write_mode(command, dest, contents, SECRET_MODE)
}

#[cfg(test)]
mod tests {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn mode_of(path: &Path) -> u32 {
        mode_bits(&std::fs::metadata(path).unwrap())
    }

    #[test]
    fn place_writes_a_missing_destination_with_a_private_mode() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("nested/.netrc");

        let outcome = place("apply", ConflictPolicy::Error, b"secret", &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read(&dest).unwrap(), b"secret");
        assert_eq!(mode_of(&dest), SECRET_MODE);
    }

    #[test]
    fn place_leaves_a_matching_destination_alone() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        place("apply", ConflictPolicy::Overwrite, b"secret", &dest).unwrap();

        let outcome = place("apply", ConflictPolicy::Error, b"secret", &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn place_narrows_a_widened_mode_back() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        std::fs::write(&dest, "secret").unwrap();
        std::fs::set_permissions(&dest, Permissions::from_mode(0o644)).unwrap();

        let outcome = place("apply", ConflictPolicy::Overwrite, b"secret", &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(mode_of(&dest), SECRET_MODE);
    }

    #[test]
    fn place_follows_the_conflict_policy() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        std::fs::write(&dest, "stale").unwrap();

        assert_eq!(
            place("apply", ConflictPolicy::Skip, b"secret", &dest).unwrap(),
            SyncOutcome::Skipped
        );
        assert_eq!(std::fs::read(&dest).unwrap(), b"stale");

        let err = place("apply", ConflictPolicy::Error, b"secret", &dest)
            .unwrap_err()
            .to_string();
        assert!(err.contains("apply: "));
        assert!(err.contains("already exists"));

        assert_eq!(
            place("apply", ConflictPolicy::Overwrite, b"secret", &dest).unwrap(),
            SyncOutcome::Replaced
        );
        assert_eq!(std::fs::read(&dest).unwrap(), b"secret");
    }

    #[test]
    fn place_replaces_a_symlink_with_a_plain_copy() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("cipher");
        let dest = dir.path().join(".netrc");
        std::fs::write(&source, "secret").unwrap();
        std::os::unix::fs::symlink(&source, &dest).unwrap();

        let outcome = place("apply", ConflictPolicy::Overwrite, b"secret", &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert!(!std::fs::symlink_metadata(&dest).unwrap().is_symlink());
        assert_eq!(std::fs::read_to_string(&source).unwrap(), "secret");
    }

    #[test]
    fn place_refuses_to_replace_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        std::fs::create_dir(&dest).unwrap();

        let err = place("apply", ConflictPolicy::Overwrite, b"secret", &dest)
            .unwrap_err()
            .to_string();

        assert!(err.contains("refusing to replace directory"));
    }

    #[test]
    fn plain_status_reads_what_the_destination_holds() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");

        assert_eq!(
            plain_status("status", b"secret", &dest).unwrap(),
            FileStatus::Missing
        );

        place("apply", ConflictPolicy::Overwrite, b"secret", &dest).unwrap();
        assert_eq!(
            plain_status("status", b"secret", &dest).unwrap(),
            FileStatus::Synced
        );

        std::fs::write(&dest, "handwritten").unwrap();
        assert_eq!(
            plain_status("status", b"secret", &dest).unwrap(),
            FileStatus::Differs
        );
    }

    #[test]
    fn a_widened_mode_reads_as_diverged() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        place("apply", ConflictPolicy::Overwrite, b"secret", &dest).unwrap();
        std::fs::set_permissions(&dest, Permissions::from_mode(0o644)).unwrap();

        assert_eq!(
            plain_status("status", b"secret", &dest).unwrap(),
            FileStatus::Differs
        );
    }
}
