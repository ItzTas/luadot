use std::path::Path;

use anyhow::Result;

use super::constants::COMMAND;
use super::fs::exists;
use super::placement::Placement;
use super::sync::{already_synced, same_contents};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Synced,
    Missing,
    Unlinked,
    Differs,
    Unreadable,
}

impl FileStatus {
    pub fn name(self) -> &'static str {
        match self {
            Self::Synced => "synced",
            Self::Missing => "missing",
            Self::Unlinked => "unlinked",
            Self::Differs => "differs",
            Self::Unreadable => "unreadable",
        }
    }
}

pub fn file_status(placement: Placement, source: &Path, dest: &Path) -> Result<FileStatus> {
    if !exists(COMMAND, dest)? {
        return Ok(FileStatus::Missing);
    }

    if already_synced(placement.link(), source, dest)? {
        return Ok(match placement.carried_by(dest) {
            true => FileStatus::Synced,
            false => FileStatus::Differs,
        });
    }

    if same_contents(source, dest)? {
        return Ok(FileStatus::Unlinked);
    }

    Ok(FileStatus::Differs)
}

#[cfg(test)]
mod tests {
    use super::super::LinkMode;
    use super::*;

    fn write(path: &Path, contents: &str) {
        std::fs::write(path, contents).unwrap();
    }

    fn placed(link: LinkMode) -> Placement<'static> {
        Placement::new(link)
    }

    #[test]
    fn reports_unlinked_on_matching_content() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        write(&dest, "data");

        assert_eq!(
            file_status(placed(LinkMode::Hard), &source, &dest).unwrap(),
            FileStatus::Unlinked
        );
    }

    #[test]
    fn copy_mode_reports_synced_by_content() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        write(&dest, "data");

        assert_eq!(
            file_status(placed(LinkMode::Copy), &source, &dest).unwrap(),
            FileStatus::Synced
        );

        write(&dest, "other");
        assert_eq!(
            file_status(placed(LinkMode::Copy), &source, &dest).unwrap(),
            FileStatus::Differs
        );
    }

    #[test]
    fn a_drifted_mode_reports_differs() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o644)).unwrap();
        std::fs::hard_link(&source, &dest).unwrap();

        let placement = placed(LinkMode::Hard).with_mode(Some(0o600));
        assert_eq!(
            file_status(placement, &source, &dest).unwrap(),
            FileStatus::Differs
        );

        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            file_status(placement, &source, &dest).unwrap(),
            FileStatus::Synced
        );
    }
}
