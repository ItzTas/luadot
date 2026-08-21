use std::path::Path;

use anyhow::{Context, Result};

use super::constants::COMMAND;
use super::fs::{create_parent, exists, mode_bits, regular_file, remove_existing, write_mode};
use super::placement::Placement;
use super::status::FileStatus;
use super::sync::{ConflictPolicy, SyncOutcome, refused};

pub fn text_status(dest: &Path, contents: &str, mode: Option<u32>) -> Result<FileStatus> {
    if !exists(COMMAND, dest)? {
        return Ok(FileStatus::Missing);
    }

    if holds(dest, contents, mode)? {
        return Ok(FileStatus::Synced);
    }

    Ok(FileStatus::Differs)
}

pub fn write_file(
    policy: ConflictPolicy,
    placement: Placement,
    dest: &Path,
    contents: &str,
) -> Result<SyncOutcome> {
    if !exists(COMMAND, dest)? {
        create_parent(COMMAND, dest)?;
        write(placement, dest, contents)?;
        return Ok(SyncOutcome::Created);
    }

    if holds(dest, contents, placement.mode())? {
        return Ok(SyncOutcome::AlreadySynced);
    }

    if let Some(outcome) = refused(COMMAND, policy, dest)? {
        return Ok(outcome);
    }

    remove_existing(COMMAND, dest)?;
    write(placement, dest, contents)?;

    Ok(SyncOutcome::Replaced)
}

fn holds(path: &Path, contents: &str, mode: Option<u32>) -> Result<bool> {
    let Some(meta) = regular_file(path) else {
        return Ok(false);
    };
    if mode.is_some_and(|mode| mode_bits(&meta) != mode) {
        return Ok(false);
    }

    Ok(matches!(std::fs::read(path), Ok(found) if found == contents.as_bytes()))
}

fn write(placement: Placement, dest: &Path, contents: &str) -> Result<()> {
    match placement.mode() {
        Some(mode) => write_mode(COMMAND, dest, contents.as_bytes(), mode)?,
        None => std::fs::write(dest, contents)
            .with_context(|| format!("{COMMAND}: failed to write {}", dest.display()))?,
    }

    placement.own(COMMAND, dest)
}

#[cfg(test)]
mod tests {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn with_mode(mode: u32) -> Placement<'static> {
        Placement::default().with_mode(Some(mode))
    }

    #[test]
    fn skip_leaves_the_destination_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::write(&dest, "stale").unwrap();

        let outcome = write_file(
            ConflictPolicy::Skip,
            Placement::default(),
            &dest,
            "generated",
        )
        .unwrap();

        assert_eq!(outcome, SyncOutcome::Skipped);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "stale");
    }

    #[test]
    fn a_mode_lands_on_the_file_it_creates() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");

        let outcome =
            write_file(ConflictPolicy::Overwrite, with_mode(0o600), &dest, "secret").unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(mode_bits(&std::fs::metadata(&dest).unwrap()), 0o600);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "secret");
    }

    #[test]
    fn a_file_the_umask_widened_is_narrowed_back() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        std::fs::write(&dest, "secret").unwrap();
        std::fs::set_permissions(&dest, Permissions::from_mode(0o644)).unwrap();

        let outcome =
            write_file(ConflictPolicy::Overwrite, with_mode(0o600), &dest, "secret").unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(mode_bits(&std::fs::metadata(&dest).unwrap()), 0o600);
    }

    #[test]
    fn text_status_reports_a_mode_that_diverges() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".netrc");
        write_file(ConflictPolicy::Overwrite, with_mode(0o644), &dest, "secret").unwrap();

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
}
