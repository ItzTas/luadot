use std::path::Path;

use anyhow::{Context, Result};
use tracing::debug;

use super::constants::COMMAND;
use super::fs::link_target;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkMode {
    #[default]
    Hard,
    Symbolic,
    Copy,
}

impl LinkMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Hard => "hard",
            Self::Symbolic => "symbolic",
            Self::Copy => "copy",
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
        LinkMode::Copy => copy(source, dest),
    }
}

fn hard(source: &Path, dest: &Path) -> Result<()> {
    match std::fs::hard_link(source, dest) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::CrossesDevices => copy(source, dest),
        Err(err) => Err(err).with_context(|| {
            format!(
                "files: failed to hard link {} -> {}",
                dest.display(),
                source.display()
            )
        }),
    }
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

fn copy(source: &Path, dest: &Path) -> Result<()> {
    let failed = || {
        format!(
            "files: failed to copy {} -> {}",
            source.display(),
            dest.display()
        )
    };

    if let (_, Some(target)) = link_target(COMMAND, source)? {
        return symbolic(&target, dest);
    }

    let mut reader = std::fs::File::open(source).with_context(failed)?;
    let mut writer = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dest)
        .with_context(failed)?;
    std::io::copy(&mut reader, &mut writer).with_context(failed)?;

    let permissions = reader.metadata().with_context(failed)?.permissions();
    std::fs::set_permissions(dest, permissions).with_context(failed)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_hard_link_shares_the_inode() {
        use std::os::unix::fs::MetadataExt;

        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&source, "hello").unwrap();

        link(LinkMode::Hard, &source, &dest).unwrap();

        let (a, b) = (
            std::fs::metadata(&source).unwrap(),
            std::fs::metadata(&dest).unwrap(),
        );
        assert_eq!((a.dev(), a.ino()), (b.dev(), b.ino()));
    }

    #[test]
    fn copy_carries_a_symlink_instead_of_its_target() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.txt");
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&target, "secret").unwrap();
        std::os::unix::fs::symlink(&target, &source).unwrap();

        link(LinkMode::Copy, &source, &dest).unwrap();

        assert!(
            std::fs::symlink_metadata(&dest)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read_link(&dest).unwrap(), target);
    }

    #[test]
    fn copy_preserves_the_mode() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&source, "secret").unwrap();
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o600)).unwrap();

        link(LinkMode::Copy, &source, &dest).unwrap();

        let mode = std::fs::metadata(&dest).unwrap().permissions().mode() & 0o7777;
        assert_eq!(mode, 0o600);
    }
}
