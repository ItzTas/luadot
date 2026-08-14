use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::files;
use crate::output::{self, Tone};
use crate::utils;

const PREVIEW_LIMIT: usize = 10;

const YES_FLAGS: &str = "-y or --yes";

const UNITS: [(u64, &str); 4] = [
    (86_400, "day"),
    (3_600, "hour"),
    (60, "minute"),
    (1, "second"),
];

#[derive(Debug, Args)]
pub struct RestoreArgs {
    #[arg(value_name = "BACKUP")]
    pub backup: Option<String>,
    #[arg(short, long, help = "Show the backups instead of putting one back")]
    pub list: bool,
    #[arg(short, long, help = "Put the files back without asking first")]
    pub yes: bool,
    #[arg(
        short = 'n',
        long,
        help = "Report what would be put back, writing nothing"
    )]
    pub dry_run: bool,
}

pub fn restore_cmd(args: RestoreArgs) -> Result<()> {
    let root = utils::backups_dir()?;
    let taken = taken(&root)?;
    if taken.is_empty() {
        output::note("no backup taken yet");
        return Ok(());
    }

    if args.list {
        return list(&taken);
    }

    let (stamp, dir) = chosen(&taken, args.backup.as_ref())?;
    let saved = files::collect_files("restore", dir)?;
    if saved.is_empty() {
        output::note(format!("backup {stamp} holds no file"));
        return Ok(());
    }

    let home = utils::home_dir()?;
    if args.dry_run {
        return foresee(dir, &home, &saved, stamp);
    }

    if !args.yes && !confirmed(dir, &saved, stamp)? {
        output::warn("aborted");
        return Ok(());
    }

    for file in &saved {
        put_back("restore", dir, &home, file)?;
    }

    output::note(format!(
        "restored {} file(s) from backup {stamp}",
        saved.len()
    ));

    Ok(())
}

fn list(taken: &[(u64, PathBuf)]) -> Result<()> {
    let now = utils::now()?;

    for (stamp, dir) in taken.iter().rev() {
        let count = files::collect_files("restore", dir)?.len();
        output::field(
            stamp,
            format!("{}  {count} file(s)", ago(now.saturating_sub(*stamp))),
        );
    }

    Ok(())
}

fn foresee(dir: &Path, home: &Path, saved: &[PathBuf], stamp: u64) -> Result<()> {
    for file in saved {
        output::entry(
            Tone::Warning,
            "restore",
            destination(dir, home, file)?.display(),
        );
    }

    output::note(format!(
        "would restore {} file(s) from backup {stamp}",
        saved.len()
    ));

    Ok(())
}

fn confirmed(dir: &Path, saved: &[PathBuf], stamp: u64) -> Result<bool> {
    output::line(preview(dir, saved));
    utils::confirm(
        "restore",
        &format!("Put {} file(s) of backup {stamp} back?", saved.len()),
        YES_FLAGS,
    )
}

fn preview(dir: &Path, saved: &[PathBuf]) -> String {
    let mut lines: Vec<String> = saved
        .iter()
        .take(PREVIEW_LIMIT)
        .map(|file| format!("  {}", utils::relative(dir, file).display()))
        .collect();

    if saved.len() > PREVIEW_LIMIT {
        lines.push(format!("  ... and {} more", saved.len() - PREVIEW_LIMIT));
    }

    lines.join("\n")
}

fn chosen<'a>(taken: &'a [(u64, PathBuf)], name: Option<&String>) -> Result<(u64, &'a Path)> {
    let Some(name) = name else {
        let (stamp, dir) = taken.last().context("restore: no backup taken yet")?;
        return Ok((*stamp, dir.as_path()));
    };

    let Some((stamp, dir)) = taken.iter().find(|(stamp, _)| stamp.to_string() == *name) else {
        bail!("restore: no backup named {name}; run `luadot restore list` to see them");
    };

    Ok((*stamp, dir.as_path()))
}

fn taken(root: &Path) -> Result<Vec<(u64, PathBuf)>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(root)
        .with_context(|| format!("restore: failed to read {}", root.display()))?;

    let mut taken = Vec::new();
    for entry in entries {
        let entry = entry.with_context(|| format!("restore: failed to read {}", root.display()))?;
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

fn destination(dir: &Path, home: &Path, file: &Path) -> Result<PathBuf> {
    let relative = file.strip_prefix(dir).with_context(|| {
        format!(
            "restore: {} is not inside the backup {}",
            file.display(),
            dir.display()
        )
    })?;

    Ok(home.join(relative))
}

fn put_back(command: &str, dir: &Path, home: &Path, file: &Path) -> Result<()> {
    let dest = destination(dir, home, file)?;

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("{command}: failed to create {}", parent.display()))?;
    }
    clear(command, &dest)?;

    utils::copy_entry(command, file, &dest)
}

fn clear(command: &str, dest: &Path) -> Result<()> {
    let meta = match std::fs::symlink_metadata(dest) {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("{command}: failed to inspect {}", dest.display()));
        }
    };

    if meta.file_type().is_dir() {
        bail!(
            "{command}: refusing to replace directory {} with a file",
            dest.display()
        );
    }

    std::fs::remove_file(dest)
        .with_context(|| format!("{command}: failed to remove {}", dest.display()))
}

fn ago(seconds: u64) -> String {
    for (size, name) in UNITS {
        if seconds < size {
            continue;
        }
        let count = seconds / size;
        let plural = match count {
            1 => "",
            _ => "s",
        };
        return format!("{count} {name}{plural} ago");
    }

    "just now".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ages_are_told_in_the_largest_unit_that_fits() {
        assert_eq!(ago(0), "just now");
        assert_eq!(ago(1), "1 second ago");
        assert_eq!(ago(59), "59 seconds ago");
        assert_eq!(ago(60), "1 minute ago");
        assert_eq!(ago(3_600), "1 hour ago");
        assert_eq!(ago(7_200), "2 hours ago");
        assert_eq!(ago(86_400), "1 day ago");
        assert_eq!(ago(200_000), "2 days ago");
    }

    #[test]
    fn backups_are_read_oldest_first_and_anything_else_ignored() {
        let root = tempfile::tempdir().unwrap();
        for name in ["200", "100", "notes"] {
            std::fs::create_dir(root.path().join(name)).unwrap();
        }
        std::fs::write(root.path().join("300"), "file").unwrap();

        let taken = taken(root.path()).unwrap();

        let stamps: Vec<u64> = taken.iter().map(|(stamp, _)| *stamp).collect();
        assert_eq!(stamps, [100, 200]);
    }

    #[test]
    fn a_missing_backups_directory_holds_nothing() {
        let root = tempfile::tempdir().unwrap();

        assert!(taken(&root.path().join("gone")).unwrap().is_empty());
    }

    #[test]
    fn the_most_recent_backup_is_the_default() {
        let taken = vec![
            (100, PathBuf::from("/data/backups/100")),
            (200, PathBuf::from("/data/backups/200")),
        ];

        let (stamp, dir) = chosen(&taken, None).unwrap();

        assert_eq!(stamp, 200);
        assert_eq!(dir, Path::new("/data/backups/200"));
    }

    #[test]
    fn a_named_backup_is_taken_as_written() {
        let taken = vec![
            (100, PathBuf::from("/data/backups/100")),
            (200, PathBuf::from("/data/backups/200")),
        ];

        let (stamp, _) = chosen(&taken, Some(&"100".to_string())).unwrap();

        assert_eq!(stamp, 100);
    }

    #[test]
    fn a_backup_that_does_not_exist_is_reported() {
        let taken = vec![(100, PathBuf::from("/data/backups/100"))];

        let err = chosen(&taken, Some(&"42".to_string()))
            .unwrap_err()
            .to_string();

        assert!(err.contains("restore: no backup named 42"));
    }

    #[test]
    fn a_restored_file_lands_back_on_its_own_path() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backup");
        let home = root.path().join("home");
        let saved = dir.join(".config/nvim/init.lua");
        std::fs::create_dir_all(saved.parent().unwrap()).unwrap();
        std::fs::write(&saved, "backed up").unwrap();

        put_back("restore", &dir, &home, &saved).unwrap();

        assert_eq!(
            std::fs::read_to_string(home.join(".config/nvim/init.lua")).unwrap(),
            "backed up"
        );
    }

    #[test]
    fn restoring_over_a_hard_link_leaves_the_other_file_alone() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backup");
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let saved = dir.join(".zshrc");
        let managed = repo.join(".zshrc");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::write(&saved, "handwritten").unwrap();
        std::fs::write(&managed, "managed").unwrap();
        std::fs::hard_link(&managed, home.join(".zshrc")).unwrap();

        put_back("restore", &dir, &home, &saved).unwrap();

        assert_eq!(
            std::fs::read_to_string(home.join(".zshrc")).unwrap(),
            "handwritten"
        );
        assert_eq!(std::fs::read_to_string(&managed).unwrap(), "managed");
    }

    #[test]
    fn a_directory_in_the_way_is_reported() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backup");
        let home = root.path().join("home");
        let saved = dir.join(".zshrc");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(home.join(".zshrc")).unwrap();
        std::fs::write(&saved, "handwritten").unwrap();

        let err = put_back("restore", &dir, &home, &saved)
            .unwrap_err()
            .to_string();

        assert!(err.contains("refusing to replace directory"));
    }
}
