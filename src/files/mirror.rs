use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result};

use crate::git;

use super::constants::{
    GENERATED_SIDE, MIRROR_BRANCH, MIRROR_MODE, MIRROR_PREFIX, MIRROR_TREE, REPOSITORY_SIDE,
    SYSTEM_SIDE,
};

static MIRROR: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Repository,
    Generated,
    System,
}

#[derive(Debug)]
pub struct Mirror {
    command: String,
    root: PathBuf,
}

#[derive(Debug)]
pub struct Tracked<'a> {
    mirror: &'a Mirror,
    root: PathBuf,
    pending: Vec<PathBuf>,
}

impl Side {
    pub fn dir(self) -> &'static str {
        match self {
            Self::Repository => REPOSITORY_SIDE,
            Self::Generated => GENERATED_SIDE,
            Self::System => SYSTEM_SIDE,
        }
    }
}

impl Mirror {
    pub fn open(command: &str) -> Result<Self> {
        let dir = std::env::temp_dir();
        loop {
            let name = format!(
                "{MIRROR_PREFIX}-{}-{}",
                std::process::id(),
                MIRROR.fetch_add(1, Ordering::Relaxed)
            );
            let root = dir.join(name);

            match std::fs::DirBuilder::new().mode(MIRROR_MODE).create(&root) {
                Ok(()) => {
                    return Ok(Self {
                        command: command.to_string(),
                        root,
                    });
                }
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(err) => {
                    return Err(err).with_context(|| {
                        format!("{command}: failed to create {}", root.display())
                    });
                }
            }
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn place(&self, side: Side, relative: &Path, contents: &[u8], mode: u32) -> Result<()> {
        written(
            &self.command,
            &self.root.join(side.dir()).join(relative),
            contents,
            mode,
        )
    }

    pub fn tracked(&self) -> Result<Tracked<'_>> {
        let root = self.root.join(MIRROR_TREE);
        std::fs::DirBuilder::new()
            .mode(MIRROR_MODE)
            .create(&root)
            .with_context(|| format!("{}: failed to create {}", self.command, root.display()))?;

        git::scratch(&self.command, &root, MIRROR_BRANCH)?;

        Ok(Tracked {
            mirror: self,
            root,
            pending: Vec::new(),
        })
    }
}

impl Tracked<'_> {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn write(&mut self, relative: &Path, contents: &[u8], mode: u32) -> Result<()> {
        written(
            &self.mirror.command,
            &self.root.join(relative),
            contents,
            mode,
        )?;
        self.pending.push(relative.to_path_buf());

        Ok(())
    }

    pub fn erase(&self, relative: &Path) -> Result<()> {
        let path = self.root.join(relative);
        match std::fs::remove_file(&path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other.with_context(|| {
                format!(
                    "{}: failed to remove {}",
                    self.mirror.command,
                    path.display()
                )
            }),
        }
    }

    pub fn stage(&mut self) -> Result<()> {
        let pending = std::mem::take(&mut self.pending);

        git::record(&self.mirror.command, &self.root, &pending)
    }
}

fn written(command: &str, path: &Path, contents: &[u8], mode: u32) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("{command}: failed to create {}", parent.display()))?;
    }

    let failed = || format!("{command}: failed to write {}", path.display());
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(path)
        .with_context(failed)?
        .write_all(contents)
        .with_context(failed)?;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .with_context(|| format!("{command}: failed to set the mode of {}", path.display()))
}

impl Drop for Mirror {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[cfg(test)]
mod tests {
    use super::super::constants::MODE_BITS;
    use super::*;

    fn bits(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & MODE_BITS
    }

    #[test]
    fn each_side_keeps_the_managed_path() {
        let mirror = Mirror::open("diff").unwrap();
        let relative = Path::new(".config/nvim/init.lua");
        mirror
            .place(Side::Repository, relative, b"managed\n", 0o644)
            .unwrap();
        mirror
            .place(Side::System, relative, b"handwritten\n", 0o644)
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(mirror.root().join("repository").join(relative)).unwrap(),
            "managed\n"
        );
        assert_eq!(
            std::fs::read_to_string(mirror.root().join("system").join(relative)).unwrap(),
            "handwritten\n"
        );
    }

    #[test]
    fn the_root_is_private_and_temporary() {
        let mirror = Mirror::open("diff").unwrap();
        let root = mirror.root().to_path_buf();
        mirror
            .place(Side::System, Path::new(".netrc"), b"secret\n", 0o600)
            .unwrap();

        assert_eq!(bits(&root), MIRROR_MODE);

        drop(mirror);
        assert!(!root.exists());
    }
}
