use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use super::constants::BACKUPS_DIR;
use crate::utils::{data_dir, expand, home_dir};

pub fn backups_root(configured: Option<&Path>) -> Result<PathBuf> {
    let Some(dir) = configured else {
        return Ok(data_dir()?.join(BACKUPS_DIR));
    };

    Ok(expand(&home_dir()?, dir))
}

pub fn now() -> Result<u64> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("the clock is set before the unix epoch")?
        .as_millis();

    u64::try_from(millis).context("the clock is too far ahead of the unix epoch")
}

pub(super) fn free(root: &Path, stamp: u64) -> PathBuf {
    let mut stamp = stamp;
    loop {
        let dir = root.join(stamp.to_string());
        if std::fs::symlink_metadata(&dir).is_err() {
            return dir;
        }
        stamp += 1;
    }
}

pub fn taken(command: &str, root: &Path) -> Result<Vec<(u64, PathBuf)>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(root)
        .with_context(|| format!("{command}: failed to read {}", root.display()))?;

    let mut taken = Vec::new();
    for entry in entries {
        let entry =
            entry.with_context(|| format!("{command}: failed to read {}", root.display()))?;
        if !entry.path().is_dir() {
            continue;
        }
        let Some(stamp) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse().ok())
        else {
            continue;
        };
        taken.push((stamp, entry.path()));
    }

    taken.sort();

    Ok(taken)
}

pub(super) fn prune(command: &str, root: &Path, keep: u32) -> Result<u32> {
    let mut taken = taken(command, root)?;
    let extra = taken.len().saturating_sub(keep as usize);

    for (_, dir) in taken.drain(..extra) {
        std::fs::remove_dir_all(&dir)
            .with_context(|| format!("{command}: failed to remove {}", dir.display()))?;
    }

    Ok(extra as u32)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_landing_on_a_taken_name_takes_the_next_free_one() {
        let root = tempfile::tempdir().unwrap();
        for name in ["100", "101"] {
            std::fs::create_dir(root.path().join(name)).unwrap();
        }

        assert_eq!(free(root.path(), 100), root.path().join("102"));
        assert_eq!(free(root.path(), 200), root.path().join("200"));
    }

    #[test]
    fn backups_are_read_oldest_first_and_anything_else_ignored() {
        let root = tempfile::tempdir().unwrap();
        for name in ["200", "100", "notes"] {
            std::fs::create_dir(root.path().join(name)).unwrap();
        }
        std::fs::write(root.path().join("300"), "file").unwrap();

        let taken = taken("restore", root.path()).unwrap();

        let stamps: Vec<u64> = taken.iter().map(|(stamp, _)| *stamp).collect();
        assert_eq!(stamps, [100, 200]);
    }

    #[test]
    fn a_missing_backups_directory_holds_nothing() {
        let root = tempfile::tempdir().unwrap();

        assert!(
            taken("restore", &root.path().join("gone"))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn pruning_keeps_the_most_recent_backups() {
        let root = tempfile::tempdir().unwrap();
        for name in ["100", "200", "300"] {
            std::fs::create_dir(root.path().join(name)).unwrap();
            std::fs::write(root.path().join(name).join(".zshrc"), name).unwrap();
        }

        assert_eq!(prune("apply", root.path(), 2).unwrap(), 1);

        assert!(!root.path().join("100").exists());
        assert!(root.path().join("200").is_dir());
        assert!(root.path().join("300").is_dir());
    }

    #[test]
    fn pruning_removes_nothing_while_the_limit_is_not_reached() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("100")).unwrap();

        assert_eq!(prune("apply", root.path(), 2).unwrap(), 0);
        assert_eq!(prune("apply", &root.path().join("gone"), 1).unwrap(), 0);
        assert!(root.path().join("100").is_dir());
    }

    #[test]
    fn a_configured_directory_replaces_the_default_one() {
        let home = home_dir().unwrap();

        assert_eq!(
            backups_root(Some(Path::new("~/dots/backups"))).unwrap(),
            home.join("dots/backups")
        );
        assert_eq!(
            backups_root(Some(Path::new("/data/backups"))).unwrap(),
            PathBuf::from("/data/backups")
        );
        assert_eq!(
            backups_root(None).unwrap(),
            data_dir().unwrap().join(BACKUPS_DIR)
        );
    }
}
