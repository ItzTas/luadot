use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::retention::Retention;
use super::store::{backups_root, copy_entry, free, now, prune};
use crate::output;
use crate::utils::managed_relative;

#[derive(Debug)]
pub struct Backup {
    command: String,
    home: PathBuf,
    root: PathBuf,
    dir: PathBuf,
    saved: u32,
    retention: Retention,
}

impl Backup {
    pub fn open(
        command: &str,
        home: &Path,
        configured: Option<&Path>,
        retention: Retention,
    ) -> Result<Self> {
        let root = backups_root(configured)?;
        let dir = free(&root, now()?);

        Ok(Self::at(command, home, dir).keeping(retention))
    }

    pub fn keeping(self, retention: Retention) -> Self {
        Self { retention, ..self }
    }

    pub fn at(command: &str, home: &Path, dir: PathBuf) -> Self {
        Self {
            command: command.to_string(),
            home: home.to_path_buf(),
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
        let relative = managed_relative(&self.home, path)
            .with_context(|| format!("{}: failed to back up {}", self.command, path.display()))?;

        let target = self.dir.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("{}: failed to create {}", self.command, parent.display())
            })?;
        }

        match copy_entry(&self.command, path, &target) {
            Err(err) if crate::files::permission_denied(&err) => {
                self.save_escalated(path, &target)?;
            }
            other => other?,
        }
        self.saved += 1;

        Ok(())
    }

    fn save_escalated(&self, path: &Path, target: &Path) -> Result<()> {
        let bytes = crate::files::escalated_read(&self.command, path)?;
        std::fs::write(target, bytes)
            .with_context(|| format!("{}: failed to write {}", self.command, target.display()))?;

        let permissions = std::fs::metadata(path)
            .with_context(|| format!("{}: failed to inspect {}", self.command, path.display()))?
            .permissions();
        std::fs::set_permissions(target, permissions).with_context(|| {
            format!(
                "{}: failed to set the mode of {}",
                self.command,
                target.display()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_file_keeps_its_path_under_the_home_prefix() {
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
            std::fs::read_to_string(dir.join("home/.config/nvim/init.lua")).unwrap(),
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

        assert_eq!(std::fs::read_link(dir.join("home/.zshrc")).unwrap(), target);
    }

    #[test]
    fn a_file_outside_the_home_directory_lands_under_the_root_prefix() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let dir = root.path().join("backup");
        let file = root.path().join("etc/hosts");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "system").unwrap();

        let mut backup = Backup::at("alt", &home, dir.clone());
        backup.save(&file).unwrap();

        assert_eq!(backup.saved(), 1);
        let saved = dir.join(managed_relative(&home, &file).unwrap());
        assert_eq!(std::fs::read_to_string(saved).unwrap(), "system");
    }

    #[test]
    fn a_backup_without_a_limit_prunes_nothing() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let dir = root.path().join("backups/100");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(root.path().join("backups/200")).unwrap();

        Backup::at("apply", &home, dir).finish().unwrap();

        assert!(root.path().join("backups/100").is_dir());
        assert!(root.path().join("backups/200").is_dir());
    }

    #[test]
    fn an_age_limit_drops_the_backups_it_has_outlived() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let dir = root.path().join("backups/100");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(root.path().join("backups/200")).unwrap();

        Backup::at("apply", &home, dir)
            .keeping(Retention::new(None, Some(60)))
            .finish()
            .unwrap();

        assert!(!root.path().join("backups/100").exists());
        assert!(!root.path().join("backups/200").exists());
    }

    #[test]
    fn an_age_limit_leaves_a_backup_it_has_not_outlived() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let dir = root.path().join("backups").join(now().unwrap().to_string());
        std::fs::create_dir_all(&dir).unwrap();

        Backup::at("apply", &home, dir.clone())
            .keeping(Retention::new(None, Some(60)))
            .finish()
            .unwrap();

        assert!(dir.is_dir());
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
