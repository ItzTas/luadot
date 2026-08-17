use std::fs::{Metadata, OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use anyhow::{Context, Result, bail};

use super::backend::Backend;
use super::constants::{MODE_BITS, SECRET_MODE};
use super::run;
use crate::files::{ConflictPolicy, FileStatus, SyncOutcome};

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

    match policy {
        ConflictPolicy::Skip => return Ok(SyncOutcome::Skipped),
        ConflictPolicy::Error => bail!("{command}: {} already exists", dest.display()),
        ConflictPolicy::Overwrite => {}
    }

    remove_existing(command, dest)?;
    write(command, dest, contents)?;

    Ok(SyncOutcome::Replaced)
}

fn holds(dest: &Path, expected: &[u8]) -> bool {
    let Ok(meta) = std::fs::symlink_metadata(dest) else {
        return false;
    };
    if !meta.file_type().is_file() || bits(&meta) != SECRET_MODE {
        return false;
    }

    matches!(std::fs::read(dest), Ok(found) if found == expected)
}

fn bits(meta: &Metadata) -> u32 {
    meta.permissions().mode() & MODE_BITS
}

fn exists(command: &str, path: &Path) -> Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => {
            Err(err).with_context(|| format!("{command}: failed to inspect {}", path.display()))
        }
    }
}

fn create_parent(command: &str, path: &Path) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)
        .with_context(|| format!("{command}: failed to create {}", parent.display()))
}

fn remove_existing(command: &str, path: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)
        .with_context(|| format!("{command}: failed to inspect {}", path.display()))?;
    if meta.file_type().is_dir() {
        bail!(
            "{command}: refusing to replace directory {} with a file",
            path.display()
        );
    }
    std::fs::remove_file(path)
        .with_context(|| format!("{command}: failed to remove {}", path.display()))
}

fn write(command: &str, dest: &Path, contents: &[u8]) -> Result<()> {
    let failed = || format!("{command}: failed to write {}", dest.display());

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(SECRET_MODE)
        .open(dest)
        .with_context(failed)?
        .write_all(contents)
        .with_context(failed)?;

    std::fs::set_permissions(dest, Permissions::from_mode(SECRET_MODE))
        .with_context(|| format!("{command}: failed to set the mode of {}", dest.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode_of(path: &Path) -> u32 {
        bits(&std::fs::metadata(path).unwrap())
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
