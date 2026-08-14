use std::path::Path;

use anyhow::{Context, Result};

use super::LinkMode;
use super::sync::{already_synced, exists};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Synced,
    Missing,
    Unlinked,
    Differs,
}

pub fn file_status(mode: LinkMode, source: &Path, dest: &Path) -> Result<FileStatus> {
    if !exists(dest)? {
        return Ok(FileStatus::Missing);
    }

    if already_synced(mode, source, dest)? {
        return Ok(FileStatus::Synced);
    }

    if same_contents(source, dest)? {
        return Ok(FileStatus::Unlinked);
    }

    Ok(FileStatus::Differs)
}

fn same_contents(source: &Path, dest: &Path) -> Result<bool> {
    let expected = std::fs::read(source)
        .with_context(|| format!("files: failed to read {}", source.display()))?;

    Ok(matches!(std::fs::read(dest), Ok(found) if found == expected))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, contents: &str) {
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn reports_missing_when_the_destination_does_not_exist() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        write(&source, "data");

        let status = file_status(LinkMode::Hard, &source, &dir.path().join("dest")).unwrap();

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
            file_status(LinkMode::Hard, &source, &dest).unwrap(),
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
            file_status(LinkMode::Symbolic, &source, &dest).unwrap(),
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
            file_status(LinkMode::Hard, &source, &dest).unwrap(),
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
            file_status(LinkMode::Hard, &source, &dest).unwrap(),
            FileStatus::Differs
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
            file_status(LinkMode::Hard, &source, &dest).unwrap(),
            FileStatus::Differs
        );
    }

    #[test]
    fn errors_when_the_source_cannot_be_read() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        std::fs::create_dir(&source).unwrap();
        write(&dest, "system");

        assert!(file_status(LinkMode::Hard, &source, &dest).is_err());
    }
}
