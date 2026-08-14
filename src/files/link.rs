use std::path::Path;

use anyhow::{Context, Result};
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkMode {
    #[default]
    Hard,
    Symbolic,
}

impl LinkMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Hard => "hard",
            Self::Symbolic => "symbolic",
        }
    }
}

pub fn link(mode: LinkMode, source: &Path, dest: &Path) -> Result<()> {
    debug!(
        mode = mode.name(),
        source = %source.display(),
        dest = %dest.display(),
        "linking"
    );
    match mode {
        LinkMode::Hard => hard(source, dest),
        LinkMode::Symbolic => symbolic(source, dest),
    }
}

fn hard(source: &Path, dest: &Path) -> Result<()> {
    std::fs::hard_link(source, dest).with_context(|| {
        format!(
            "files: failed to hard link {} -> {}",
            dest.display(),
            source.display()
        )
    })
}

fn symbolic(source: &Path, dest: &Path) -> Result<()> {
    std::os::unix::fs::symlink(source, dest).with_context(|| {
        format!(
            "files: failed to symlink {} -> {}",
            dest.display(),
            source.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_hard() {
        assert_eq!(LinkMode::default(), LinkMode::Hard);
    }

    #[test]
    fn hard_link_shares_the_same_inode() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&source, "hello").unwrap();

        link(LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "hello");
        std::fs::write(&source, "changed").unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "changed");
    }

    #[test]
    fn symbolic_link_points_at_the_source() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&source, "hello").unwrap();

        link(LinkMode::Symbolic, &source, &dest).unwrap();

        let kind = std::fs::symlink_metadata(&dest).unwrap().file_type();
        assert!(kind.is_symlink());
        assert_eq!(std::fs::read_link(&dest).unwrap(), source);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "hello");
    }

    #[test]
    fn fails_when_dest_already_exists() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&source, "hello").unwrap();
        std::fs::write(&dest, "existing").unwrap();

        assert!(link(LinkMode::Hard, &source, &dest).is_err());
    }
}
