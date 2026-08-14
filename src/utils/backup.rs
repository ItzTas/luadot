use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use super::constants::BACKUPS_DIR;
use super::paths::data_dir;
use crate::output;

#[derive(Debug)]
pub struct Backup {
    command: String,
    home: PathBuf,
    dir: PathBuf,
    saved: u32,
}

pub fn backups_dir() -> Result<PathBuf> {
    Ok(data_dir()?.join(BACKUPS_DIR))
}

pub fn now() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("the clock is set before the unix epoch")?
        .as_secs())
}

pub fn copy_entry(command: &str, source: &Path, dest: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(source)
        .with_context(|| format!("{command}: failed to inspect {}", source.display()))?;

    if meta.file_type().is_symlink() {
        let target = std::fs::read_link(source)
            .with_context(|| format!("{command}: failed to read {}", source.display()))?;
        return std::os::unix::fs::symlink(&target, dest).with_context(|| {
            format!(
                "{command}: failed to link {} -> {}",
                dest.display(),
                target.display()
            )
        });
    }

    std::fs::copy(source, dest).map(|_| ()).with_context(|| {
        format!(
            "{command}: failed to copy {} -> {}",
            source.display(),
            dest.display()
        )
    })
}

impl Backup {
    pub fn open(command: &str, home: &Path) -> Result<Self> {
        let dir = backups_dir()?.join(now()?.to_string());
        Ok(Self::at(command, home, dir))
    }

    pub fn at(command: &str, home: &Path, dir: PathBuf) -> Self {
        Self {
            command: command.to_string(),
            home: home.to_path_buf(),
            dir,
            saved: 0,
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn saved(&self) -> u32 {
        self.saved
    }

    pub fn report(&self) {
        if self.saved == 0 {
            return;
        }

        output::note(format!(
            "backed up {} replaced file(s) in {}",
            self.saved,
            self.dir.display()
        ));
    }

    pub fn save(&mut self, path: &Path) -> Result<()> {
        let Ok(relative) = path.strip_prefix(&self.home) else {
            output::warn(format!(
                "{}: {} is outside {} and was not backed up",
                self.command,
                path.display(),
                self.home.display()
            ));
            return Ok(());
        };

        let target = self.dir.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("{}: failed to create {}", self.command, parent.display())
            })?;
        }

        copy_entry(&self.command, path, &target)?;
        self.saved += 1;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_file_keeps_its_path_below_the_home_directory() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let dir = root.path().join("backup");
        let file = home.join(".config/nvim/init.lua");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "system").unwrap();

        let mut backup = Backup::at("apply", &home, dir.clone());
        backup.save(&file).unwrap();

        assert_eq!(backup.saved(), 1);
        assert_eq!(
            std::fs::read_to_string(dir.join(".config/nvim/init.lua")).unwrap(),
            "system"
        );
    }

    #[test]
    fn a_symlink_is_saved_as_a_symlink() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let dir = root.path().join("backup");
        let target = root.path().join("elsewhere");
        let file = home.join(".zshrc");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(&target, "data").unwrap();
        std::os::unix::fs::symlink(&target, &file).unwrap();

        Backup::at("apply", &home, dir.clone()).save(&file).unwrap();

        assert_eq!(std::fs::read_link(dir.join(".zshrc")).unwrap(), target);
    }

    #[test]
    fn a_file_outside_the_home_directory_is_left_out() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let dir = root.path().join("backup");
        let file = root.path().join("etc/hosts");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "system").unwrap();

        let mut backup = Backup::at("alt", &home, dir.clone());
        backup.save(&file).unwrap();

        assert_eq!(backup.saved(), 0);
        assert!(!dir.exists());
    }

    #[test]
    fn nothing_is_created_until_a_file_is_saved() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backup");

        let backup = Backup::at("apply", root.path(), dir.clone());

        assert_eq!(backup.dir(), dir);
        assert!(!dir.exists());
    }
}
