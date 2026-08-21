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
    fn reports_missing_when_the_destination_does_not_exist() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        write(&source, "data");

        let status =
            file_status(placed(LinkMode::Hard), &source, &dir.path().join("dest")).unwrap();

        assert_eq!(status, FileStatus::Missing);
    }

    #[test]
    fn reports_synced_when_hard_linked() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        std::fs::hard_link(&source, &dest).unwrap();

        assert_eq!(
            file_status(placed(LinkMode::Hard), &source, &dest).unwrap(),
            FileStatus::Synced
        );
    }

    #[test]
    fn reports_synced_when_the_symlink_points_at_the_source() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        std::os::unix::fs::symlink(&source, &dest).unwrap();

        assert_eq!(
            file_status(placed(LinkMode::Symbolic), &source, &dest).unwrap(),
            FileStatus::Synced
        );
    }

    #[test]
    fn reports_unlinked_when_the_contents_match_but_the_link_does_not() {
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
    fn reports_differs_when_the_contents_diverge() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "repo");
        write(&dest, "system");

        assert_eq!(
            file_status(placed(LinkMode::Hard), &source, &dest).unwrap(),
            FileStatus::Differs
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
    fn a_mode_that_drifted_reports_differs_even_when_linked() {
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

    #[test]
    fn reports_differs_for_a_dangling_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        std::os::unix::fs::symlink(dir.path().join("gone"), &dest).unwrap();

        assert_eq!(
            file_status(placed(LinkMode::Hard), &source, &dest).unwrap(),
            FileStatus::Differs
        );
    }
}
