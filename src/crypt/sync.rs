use std::path::Path;

use anyhow::Result;

use super::backend::Backend;
use super::constants::SECRET_MODE;
use super::lock::Lock;
use super::run;
use crate::files::{
    ConflictPolicy, FileStatus, Placement, SyncOutcome, create_parent, exists, mode_bits, refused,
    regular_file, remove_existing, write_mode,
};

pub fn status(
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

    plain_status(command, &expected, dest, mode)
}

pub fn plain_status(
    command: &str,
    expected: &[u8],
    dest: &Path,
    mode: Option<u32>,
) -> Result<FileStatus> {
    if !exists(command, dest)? {
        return Ok(FileStatus::Missing);
    }

    if holds(dest, expected, secret_mode(mode)) {
        return Ok(FileStatus::Synced);
    }

    Ok(FileStatus::Differs)
}

pub fn place(
    command: &str,
    policy: ConflictPolicy,
    placement: Placement,
    contents: &[u8],
    dest: &Path,
) -> Result<SyncOutcome> {
    if !exists(command, dest)? {
        create_parent(command, dest)?;
        write(command, placement, dest, contents)?;
        return Ok(SyncOutcome::Created);
    }

    if holds(dest, contents, secret_mode(placement.mode())) {
        return Ok(SyncOutcome::AlreadySynced);
    }

    if let Some(outcome) = refused(command, policy, dest)? {
        return Ok(outcome);
    }

    remove_existing(command, dest)?;
    write(command, placement, dest, contents)?;

    Ok(SyncOutcome::Replaced)
}

fn secret_mode(mode: Option<u32>) -> u32 {
    mode.unwrap_or(SECRET_MODE)
}

fn holds(dest: &Path, expected: &[u8], mode: u32) -> bool {
    let Some(meta) = regular_file(dest) else {
        return false;
    };
    if mode_bits(&meta) != mode {
        return false;
    }

    matches!(std::fs::read(dest), Ok(found) if found == expected)
}

fn write(command: &str, placement: Placement, dest: &Path, contents: &[u8]) -> Result<()> {
    write_mode(command, dest, contents, secret_mode(placement.mode()))?;

    placement.own(command, dest)
}

#[cfg(test)]
mod tests {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn mode_of(path: &Path) -> u32 {
        mode_bits(&std::fs::metadata(path).unwrap())
    }

    fn placed(mode: Option<u32>) -> Placement<'static> {
        Placement::default().with_mode(mode)
    }

    #[test]
    fn place_writes_a_missing_destination_with_a_private_mode() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("nested/.netrc");

        let outcome = place(
            "apply",
            ConflictPolicy::Error,
            placed(None),
            b"secret",
            &dest,
        )
        .unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read(&dest).unwrap(), b"secret");
        assert_eq!(mode_of(&dest), SECRET_MODE);
    }

    #[test]
    fn place_follows_the_conflict_policy() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        std::fs::write(&dest, "stale").unwrap();

        assert_eq!(
            place(
                "apply",
                ConflictPolicy::Skip,
                placed(None),
                b"secret",
                &dest
            )
            .unwrap(),
            SyncOutcome::Skipped
        );
        assert_eq!(std::fs::read(&dest).unwrap(), b"stale");

        let err = place(
            "apply",
            ConflictPolicy::Error,
            placed(None),
            b"secret",
            &dest,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("apply: "));
        assert!(err.contains("already exists"));

        assert_eq!(
            place(
                "apply",
                ConflictPolicy::Overwrite,
                placed(None),
                b"secret",
                &dest
            )
            .unwrap(),
            SyncOutcome::Replaced
        );
        assert_eq!(std::fs::read(&dest).unwrap(), b"secret");
    }

    #[test]
    fn plain_status_weighs_the_contents_and_the_mode() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("wg0.conf");

        assert_eq!(
            plain_status("apply", b"secret", &dest, Some(0o640)).unwrap(),
            FileStatus::Missing
        );

        place(
            "apply",
            ConflictPolicy::Error,
            placed(Some(0o640)),
            b"secret",
            &dest,
        )
        .unwrap();
        assert_eq!(
            plain_status("apply", b"secret", &dest, Some(0o640)).unwrap(),
            FileStatus::Synced
        );
        assert_eq!(
            plain_status("apply", b"secret", &dest, None).unwrap(),
            FileStatus::Differs
        );

        std::fs::set_permissions(&dest, Permissions::from_mode(SECRET_MODE)).unwrap();
        assert_eq!(
            plain_status("apply", b"handwritten", &dest, None).unwrap(),
            FileStatus::Differs
        );
    }
}
