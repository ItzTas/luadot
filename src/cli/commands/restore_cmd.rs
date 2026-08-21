use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::backup;
use crate::files;
use crate::lua;
use crate::output::{self, Tone};
use crate::utils;

const PREVIEW_LIMIT: usize = 10;

const YES_FLAGS: &str = "-y or --yes";

const MILLIS: u64 = 1_000;

#[derive(Debug, Args)]
pub struct RestoreArgs {
    #[arg(
        value_name = "BACKUP",
        help = "The backup to put back, the most recent one when left out"
    )]
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
    let config = lua::load_config()?;
    let root = backup::backups_root(utils::configured("restore", &config)?.backup_dir())?;
    let taken = backup::taken("restore", &root)?;
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

    if args.dry_run {
        return foresee(dir, &saved, stamp);
    }

    if !args.yes && !confirmed(dir, &saved, stamp)? {
        output::warn("aborted");
        return Ok(());
    }

    for file in &saved {
        put_back("restore", dir, file)?;
    }

    output::note(format!(
        "restored {} file(s) from backup {stamp}",
        saved.len()
    ));

    Ok(())
}

fn list(taken: &[(u64, PathBuf)]) -> Result<()> {
    let rows = rows(taken, backup::now()?)?;
    let width = rows
        .iter()
        .map(|(_, ago, _)| ago.chars().count())
        .max()
        .unwrap_or_default()
        + output::GAP.len();

    for (stamp, ago, count) in rows {
        output::field(
            stamp,
            format!("{}{count} file(s)", output::column(ago, width)),
        );
    }

    Ok(())
}

fn rows(taken: &[(u64, PathBuf)], now: u64) -> Result<Vec<(u64, String, usize)>> {
    taken
        .iter()
        .rev()
        .map(|(stamp, dir)| {
            let count = files::collect_files("restore", dir)?.len();
            Ok((*stamp, ago(now.saturating_sub(*stamp) / MILLIS), count))
        })
        .collect()
}

fn foresee(dir: &Path, saved: &[PathBuf], stamp: u64) -> Result<()> {
    for file in saved {
        output::entry(Tone::Warning, "restore", destination(dir, file)?.display());
    }

    output::note(format!(
        "would restore {} file(s) from backup {stamp}",
        saved.len()
    ));

    Ok(())
}

fn confirmed(dir: &Path, saved: &[PathBuf], stamp: u64) -> Result<bool> {
    output::line(preview(dir, saved));
    output::confirm(
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

fn destination(dir: &Path, file: &Path) -> Result<PathBuf> {
    let relative = file.strip_prefix(dir).with_context(|| {
        format!(
            "restore: {} cannot be placed back from the backup {}",
            file.display(),
            dir.display()
        )
    })?;

    Ok(Path::new("/").join(relative))
}

fn put_back(command: &str, dir: &Path, file: &Path) -> Result<()> {
    let dest = destination(dir, file)?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("{command}: failed to create {}", parent.display()))?;
    }
    clear(command, &dest)?;

    backup::copy_entry(command, file, &dest)
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
    match seconds {
        0 => "just now".to_string(),
        seconds => format!("{} ago", utils::span(seconds)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_backup_that_does_not_exist_is_reported() {
        let taken = vec![(100, PathBuf::from("/data/backups/100"))];

        let err = chosen(&taken, Some(&"42".to_string()))
            .unwrap_err()
            .to_string();

        assert!(err.contains("restore: no backup named 42"));
    }

    fn backed(dir: &Path, path: &Path) -> PathBuf {
        dir.join(path.strip_prefix("/").unwrap())
    }

    #[test]
    fn a_saved_file_goes_back_to_its_absolute_path() {
        let dir = Path::new("/data/backups/100");

        assert_eq!(
            destination(dir, &dir.join("home/u/.zshrc")).unwrap(),
            PathBuf::from("/home/u/.zshrc")
        );

        let err = destination(dir, Path::new("/elsewhere/.zshrc"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("cannot be placed back from the backup"));
    }

    #[test]
    fn restoring_over_a_hard_link_leaves_the_other_file_alone() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backup");
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        let saved = backed(&dir, &home.join(".zshrc"));
        let managed = repo.join(".zshrc");
        std::fs::create_dir_all(saved.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::write(&saved, "handwritten").unwrap();
        std::fs::write(&managed, "managed").unwrap();
        std::fs::hard_link(&managed, home.join(".zshrc")).unwrap();

        put_back("restore", &dir, &saved).unwrap();

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
        let saved = backed(&dir, &home.join(".zshrc"));
        std::fs::create_dir_all(saved.parent().unwrap()).unwrap();
        std::fs::create_dir_all(home.join(".zshrc")).unwrap();
        std::fs::write(&saved, "handwritten").unwrap();

        let err = put_back("restore", &dir, &saved).unwrap_err().to_string();

        assert!(err.contains("refusing to replace directory"));
    }
}
