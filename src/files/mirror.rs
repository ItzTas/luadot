use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result};

use super::constants::{MIRROR_MODE, MIRROR_PREFIX, REPOSITORY_SIDE, SYSTEM_SIDE};

static MIRROR: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Repository,
    System,
}

#[derive(Debug)]
pub struct Mirror {
    command: String,
    root: PathBuf,
}

impl Side {
    pub fn dir(self) -> &'static str {
        match self {
            Self::Repository => REPOSITORY_SIDE,
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
        let path = self.root.join(side.dir()).join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("{}: failed to create {}", self.command, parent.display())
            })?;
        }

        let failed = || format!("{}: failed to write {}", self.command, path.display());
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(mode)
            .open(&path)
            .with_context(failed)?
            .write_all(contents)
            .with_context(failed)?;

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).with_context(|| {
            format!(
                "{}: failed to set the mode of {}",
                self.command,
                path.display()
            )
        })
    }
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
    fn each_side_keeps_the_managed_path_below_its_own_directory() {
        let mirror = Mirror::open("diff").unwrap();
        let relative = Path::new("home/.config/nvim/init.lua");
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
    fn a_placed_file_carries_the_mode_it_is_given() {
        let mirror = Mirror::open("diff").unwrap();
        let relative = Path::new("root/etc/app.conf");
        mirror
            .place(Side::Repository, relative, b"conf\n", 0o640)
            .unwrap();
        mirror
            .place(Side::System, relative, b"conf\n", 0o600)
            .unwrap();

        assert_eq!(
            bits(&mirror.root().join("repository").join(relative)),
            0o640
        );
        assert_eq!(bits(&mirror.root().join("system").join(relative)), 0o600);
    }

    #[test]
    fn the_root_is_private_and_goes_away_with_the_mirror() {
        let mirror = Mirror::open("diff").unwrap();
        let root = mirror.root().to_path_buf();
        mirror
            .place(Side::System, Path::new("home/.netrc"), b"secret\n", 0o600)
            .unwrap();

        assert_eq!(bits(&root), MIRROR_MODE);

        drop(mirror);
        assert!(!root.exists());
    }

    #[test]
    fn two_mirrors_never_share_a_root() {
        let first = Mirror::open("diff").unwrap();
        let second = Mirror::open("diff").unwrap();

        assert_ne!(first.root(), second.root());
    }

    #[test]
    fn a_side_names_the_directory_it_holds() {
        assert_eq!(Side::Repository.dir(), "repository");
        assert_eq!(Side::System.dir(), "system");
    }
}
