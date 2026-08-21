use std::path::Path;

use anyhow::{Result, bail};

use super::{ConflictPolicy, FileStatus, SyncOutcome};

pub fn predict(policy: ConflictPolicy, status: FileStatus, dest: &Path) -> Result<SyncOutcome> {
    match status {
        FileStatus::Missing => Ok(SyncOutcome::Created),
        FileStatus::Synced => Ok(SyncOutcome::AlreadySynced),
        FileStatus::Unlinked | FileStatus::Differs => diverged(policy, dest),
        FileStatus::Unreadable => bail!("files: {} cannot be read", dest.display()),
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
}
