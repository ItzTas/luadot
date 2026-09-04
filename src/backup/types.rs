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

        let meta = std::fs::symlink_metadata(path)
            .with_context(|| format!("{}: failed to inspect {}", self.command, path.display()))?;
        if meta.is_dir() {
            return self.save_dir(path, &target);
        }

        copy_entry(&self.command, path, &target)?;
        self.saved += 1;

        Ok(())
    }

    fn save_dir(&mut self, path: &Path, target: &Path) -> Result<()> {
        std::fs::create_dir_all(target)
            .with_context(|| format!("{}: failed to create {}", self.command, target.display()))?;

        let entries = std::fs::read_dir(path)
            .with_context(|| format!("{}: failed to read {}", self.command, path.display()))?;
        for entry in entries {
            let entry = entry
                .with_context(|| format!("{}: failed to read {}", self.command, path.display()))?;
            self.save(&entry.path())?;
        }

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
    fn a_saved_file_keeps_its_absolute_path() {
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
    fn a_saved_directory_keeps_its_tree() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backup");
        let nvim = root.path().join("home/.config/nvim");
        std::fs::create_dir_all(nvim.join("lua")).unwrap();
        std::fs::write(nvim.join("init.lua"), "init").unwrap();
        std::fs::write(nvim.join("lua/plugins.lua"), "plugins").unwrap();

        let mut backup = Backup::at("apply", dir.clone());
        backup.save(&nvim).unwrap();

        assert_eq!(backup.saved(), 2);
        assert_eq!(
            std::fs::read_to_string(backed(&dir, &nvim.join("lua/plugins.lua"))).unwrap(),
            "plugins"
        );
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
}
