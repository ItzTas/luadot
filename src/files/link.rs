use std::path::Path;

use anyhow::{Context, Result};
use tracing::debug;

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

fn copy(source: &Path, dest: &Path) -> Result<()> {
    let failed = || {
        format!(
            "files: failed to copy {} -> {}",
            source.display(),
            dest.display()
        )
    };

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
    fn copy_preserves_the_mode_of_the_source() {
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
