use std::path::Path;

use anyhow::Result;

use super::backend::Backend;
use super::constants::SECRET_MODE;
use super::lock::Lock;
use super::run;
use crate::files::{
    ConflictPolicy, FileStatus, SyncOutcome, create_parent, exists, mode_bits, place_contents,
    read_contents, refused, regular_file, remove_existing, write_mode,
};

pub fn status(
    command: &str,
    backend: Backend,
    lock: Lock,
    identity: Option<&Path>,
    source: &Path,
    dest: &Path,
) -> Result<FileStatus> {
    if !exists(command, dest)? {
        return Ok(FileStatus::Missing);
    }

    let Ok(expected) = run::decrypt(command, backend, lock, identity, source) else {
        return Ok(FileStatus::Unreadable);
    };

    plain_status(command, &expected, dest)
}

pub fn system_status(
    command: &str,
    backend: Backend,
    lock: Lock,
    identity: Option<&Path>,
    source: &Path,
    dest: &Path,
    mode: Option<u32>,
) -> Result<FileStatus> {
    if !exists(command, dest)? {
        return Ok(FileStatus::Missing);
    }

    let Ok(expected) = run::decrypt(command, backend, lock, identity, source) else {
        return Ok(FileStatus::Unreadable);
    };
    if !carries(dest, secret_mode(mode)) {
        return Ok(FileStatus::Differs);
    }

    let Ok(found) = std::fs::read(dest) else {
        return Ok(FileStatus::Unreadable);
    };

    Ok(compared(&found, &expected))
}

pub fn escalated_status(
    command: &str,
    expected: &[u8],
    dest: &Path,
    mode: Option<u32>,
) -> Result<FileStatus> {
    if !exists(command, dest)? {
        return Ok(FileStatus::Missing);
    }
    if !carries(dest, secret_mode(mode)) {
        return Ok(FileStatus::Differs);
    }

    Ok(compared(&read_contents(command, dest)?, expected))
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

pub fn place_system(
    command: &str,
    policy: ConflictPolicy,
    contents: &[u8],
    dest: &Path,
    mode: Option<u32>,
    owner: Option<&str>,
) -> Result<SyncOutcome> {
    let outcome = match escalated_status(command, contents, dest, mode)? {
        FileStatus::Missing => SyncOutcome::Created,
        FileStatus::Synced => return Ok(SyncOutcome::AlreadySynced),
        _ => match refused(command, policy, dest)? {
            Some(outcome) => return Ok(outcome),
            None => SyncOutcome::Replaced,
        },
    };

    place_contents(command, contents, dest, secret_mode(mode), owner)?;

    Ok(outcome)
}

fn secret_mode(mode: Option<u32>) -> u32 {
    mode.unwrap_or(SECRET_MODE)
}

fn carries(dest: &Path, mode: u32) -> bool {
    matches!(regular_file(dest), Some(meta) if mode_bits(&meta) == mode)
}

fn compared(found: &[u8], expected: &[u8]) -> FileStatus {
    match found == expected {
        true => FileStatus::Synced,
        false => FileStatus::Differs,
    }
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
    fn place_system_writes_the_secret_with_the_mode_the_rules_ask_for() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("etc/wireguard/wg0.conf");

        let outcome = place_system(
            "apply",
            ConflictPolicy::Error,
            b"PrivateKey = secret\n",
            &dest,
            Some(0o640),
            None,
        )
        .unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read(&dest).unwrap(), b"PrivateKey = secret\n");
        assert_eq!(mode_of(&dest), 0o640);
    }

    #[test]
    fn place_system_follows_the_conflict_policy() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("wg0.conf");
        std::fs::write(&dest, "handwritten").unwrap();

        assert_eq!(
            place_system("apply", ConflictPolicy::Skip, b"secret", &dest, None, None).unwrap(),
            SyncOutcome::Skipped
        );
        assert_eq!(std::fs::read(&dest).unwrap(), b"handwritten");

        assert!(
            place_system("apply", ConflictPolicy::Error, b"secret", &dest, None, None).is_err()
        );

        assert_eq!(
            place_system(
                "apply",
                ConflictPolicy::Overwrite,
                b"secret",
                &dest,
                None,
                None
            )
            .unwrap(),
            SyncOutcome::Replaced
        );
        assert_eq!(std::fs::read(&dest).unwrap(), b"secret");
    }

    #[test]
    fn escalated_status_weighs_the_contents_and_the_mode() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("wg0.conf");

        assert_eq!(
            escalated_status("apply", b"secret", &dest, Some(0o640)).unwrap(),
            FileStatus::Missing
        );

        place_system(
            "apply",
            ConflictPolicy::Error,
            b"secret",
            &dest,
            Some(0o640),
            None,
        )
        .unwrap();
        assert_eq!(
            escalated_status("apply", b"secret", &dest, Some(0o640)).unwrap(),
            FileStatus::Synced
        );
        assert_eq!(
            escalated_status("apply", b"secret", &dest, None).unwrap(),
            FileStatus::Differs
        );

        std::fs::set_permissions(&dest, Permissions::from_mode(SECRET_MODE)).unwrap();
        assert_eq!(
            escalated_status("apply", b"handwritten", &dest, None).unwrap(),
            FileStatus::Differs
        );
    }

    #[test]
    fn a_system_secret_the_system_never_got_reads_as_missing() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(
            system_status(
                "status",
                Backend::Age,
                Lock::Keys,
                None,
                &dir.path().join("wg0.conf.age"),
                &dir.path().join("wg0.conf"),
                None,
            )
            .unwrap(),
            FileStatus::Missing
        );
    }
}
