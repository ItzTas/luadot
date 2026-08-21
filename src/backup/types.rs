use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::retention::Retention;
use super::store::{backups_root, copy_entry, free, now, prune};
use crate::output;

#[derive(Debug)]
pub struct Backup {
    command: String,
    root: PathBuf,
    dir: PathBuf,
    saved: u32,
    retention: Retention,
}

impl Backup {
    pub fn open(command: &str, configured: Option<&Path>, retention: Retention) -> Result<Self> {
        let root = backups_root(configured)?;
        let dir = free(&root, now()?);

        Ok(Self::at(command, dir).keeping(retention))
    }

    pub fn keeping(self, retention: Retention) -> Self {
        Self { retention, ..self }
    }

    pub fn at(command: &str, dir: PathBuf) -> Self {
        Self {
            command: command.to_string(),
            root: dir.parent().unwrap_or(&dir).to_path_buf(),
            dir,
            saved: 0,
            retention: Retention::default(),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn saved(&self) -> u32 {
        self.saved
    }

    pub fn finish(&self) -> Result<()> {
        self.report();

        if self.retention.is_empty() {
            return Ok(());
        }

        let dropped = prune(&self.command, &self.root, self.retention, now()?)?;
        if dropped > 0 {
            output::note(format!(
                "dropped {dropped} backup(s), {}",
                self.retention.label()
            ));
        }

        Ok(())
    }

    fn report(&self) {
        if self.saved > 0 {
            output::note(format!(
                "backed up {} file(s) in {}",
                self.saved,
                self.dir.display()
            ));
        }
    }

    pub fn save(&mut self, path: &Path) -> Result<()> {
        let relative = path
            .strip_prefix("/")
            .with_context(|| format!("{}: failed to back up {}", self.command, path.display()))?;

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

    fn backed(dir: &Path, path: &Path) -> PathBuf {
        dir.join(path.strip_prefix("/").unwrap())
    }

    #[test]
    fn a_saved_file_keeps_its_absolute_path_below_the_backup() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backup");
        let file = root.path().join("home/.config/nvim/init.lua");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "system").unwrap();

        let mut backup = Backup::at("apply", dir.clone());
        backup.save(&file).unwrap();

        assert_eq!(backup.saved(), 1);
        assert_eq!(
            std::fs::read_to_string(backed(&dir, &file)).unwrap(),
            "system"
        );
    }

    #[test]
    fn a_symlink_is_saved_as_a_symlink() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backup");
        let target = root.path().join("elsewhere");
        let file = root.path().join("home/.zshrc");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&target, "data").unwrap();
        std::os::unix::fs::symlink(&target, &file).unwrap();

        Backup::at("apply", dir.clone()).save(&file).unwrap();

        assert_eq!(std::fs::read_link(backed(&dir, &file)).unwrap(), target);
    }

    #[test]
    fn a_relative_path_is_refused() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backup");

        let mut backup = Backup::at("rm", dir.clone());
        let err = backup.save(Path::new("relative/.zshrc")).unwrap_err();

        assert_eq!(backup.saved(), 0);
        assert_eq!(err.to_string(), "rm: failed to back up relative/.zshrc");
        assert!(!dir.exists());
    }

    #[test]
    fn an_age_limit_drops_the_backups_it_has_outlived() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backups/100");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(root.path().join("backups/200")).unwrap();

        Backup::at("apply", dir)
            .keeping(Retention::new(None, Some(60)))
            .finish()
            .unwrap();

        assert!(!root.path().join("backups/100").exists());
        assert!(!root.path().join("backups/200").exists());
    }

    #[test]
    fn nothing_is_created_until_a_file_is_saved() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backup");

        let backup = Backup::at("apply", dir.clone());

        assert_eq!(backup.dir(), dir);
        assert!(!dir.exists());
    }
}
