use std::path::Path;

use anyhow::{Result, bail};

use super::{ConflictPolicy, FileStatus, SyncOutcome};

pub fn predict(policy: ConflictPolicy, status: FileStatus, dest: &Path) -> Result<SyncOutcome> {
    match status {
        FileStatus::Missing => Ok(SyncOutcome::Created),
        FileStatus::Synced => Ok(SyncOutcome::AlreadySynced),
        FileStatus::Unlinked | FileStatus::Differs => diverged(policy, dest),
    }
}

fn diverged(policy: ConflictPolicy, dest: &Path) -> Result<SyncOutcome> {
    match policy {
        ConflictPolicy::Overwrite => Ok(SyncOutcome::Replaced),
        ConflictPolicy::Skip => Ok(SyncOutcome::Skipped),
        ConflictPolicy::Error => bail!("files: {} already exists", dest.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dest() -> &'static Path {
        Path::new("/home/u/.bashrc")
    }

    #[test]
    fn a_missing_destination_is_created_whatever_the_policy() {
        for policy in [
            ConflictPolicy::Overwrite,
            ConflictPolicy::Skip,
            ConflictPolicy::Error,
        ] {
            assert_eq!(
                predict(policy, FileStatus::Missing, dest()).unwrap(),
                SyncOutcome::Created
            );
        }
    }

    #[test]
    fn a_synced_destination_is_left_alone_whatever_the_policy() {
        for policy in [
            ConflictPolicy::Overwrite,
            ConflictPolicy::Skip,
            ConflictPolicy::Error,
        ] {
            assert_eq!(
                predict(policy, FileStatus::Synced, dest()).unwrap(),
                SyncOutcome::AlreadySynced
            );
        }
    }

    #[test]
    fn a_diverging_destination_follows_the_policy() {
        for status in [FileStatus::Unlinked, FileStatus::Differs] {
            assert_eq!(
                predict(ConflictPolicy::Overwrite, status, dest()).unwrap(),
                SyncOutcome::Replaced
            );
            assert_eq!(
                predict(ConflictPolicy::Skip, status, dest()).unwrap(),
                SyncOutcome::Skipped
            );
        }
    }

    #[test]
    fn the_error_policy_reports_the_destination() {
        let err = predict(ConflictPolicy::Error, FileStatus::Differs, dest())
            .unwrap_err()
            .to_string();

        assert_eq!(err, "files: /home/u/.bashrc already exists");
    }
}
