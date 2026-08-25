use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::backup;
use crate::files::{self, SyncOutcome};
use crate::lua;
use crate::output;
use crate::state;
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
    #[arg(
        short,
        long,
        help = "Show the backups, or the files of the one named, instead of putting one back"
    )]
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
    let settings = utils::configured("restore", &config)?;
    let roots = managed_roots(settings.repo_dir())?;
    let root = backup::backups_root(settings.backup_dir())?;
    let taken = backup::taken("restore", &root)?;
    if taken.is_empty() {
        output::note("no backup taken yet");
        return Ok(());
    }

    if args.list {
        return list(&roots, &taken, args.backup.as_ref());
    }

    let (stamp, dir) = chosen(&taken, args.backup.as_ref())?;
    let saved = files::collect_files("restore", dir)?;
    if saved.is_empty() {
        output::note(format!("backup {stamp} holds no file"));
        return Ok(());
    }

    if args.dry_run {
        return foresee(&roots, dir, &saved, stamp);
    }

    if !args.yes && !confirmed(&roots, dir, &saved, stamp)? {
        output::warn("aborted");
        return Ok(());
    }

    let mut created = 0usize;
    for file in &saved {
        let (dest, outcome) = put_back("restore", &roots, dir, file)?;
        output::report(outcome, dest.display());
        created += usize::from(outcome == SyncOutcome::Created);
    }

    output::note(summary("restored", saved.len(), created, stamp));

    Ok(())
}

fn managed_roots(repo_dir: Option<&Path>) -> Result<Vec<PathBuf>> {
    let home = utils::home_dir()?;
    let repo = match repo_dir {
        Some(dir) => Some(utils::expand(&home, dir)),
        None => state::load()?.repo().map(Path::to_path_buf),
    };

    let mut roots = vec![home];
    roots.extend(repo.filter(|repo| utils::managed_relative(&roots[0], repo).is_err()));

    Ok(roots)
}

fn list(roots: &[PathBuf], taken: &[(u64, PathBuf)], name: Option<&String>) -> Result<()> {
    let Some(name) = name else {
        return show(taken);
    };

    let (stamp, dir) = chosen(taken, Some(name))?;
    show(&[(stamp, dir.to_path_buf())])?;

    for file in files::collect_files("restore", dir)? {
        output::hint(destination(roots, dir, &file)?.display());
    }

    Ok(())
}

fn show(taken: &[(u64, PathBuf)]) -> Result<()> {
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

fn foresee(roots: &[PathBuf], dir: &Path, saved: &[PathBuf], stamp: u64) -> Result<()> {
    let mut created = 0usize;
    for file in saved {
        let dest = destination(roots, dir, file)?;
        let outcome = planned("restore", &dest)?;
        output::preview(outcome, dest.display());
        created += usize::from(outcome == SyncOutcome::Created);
    }

    output::note(summary("would restore", saved.len(), created, stamp));

    Ok(())
}

fn confirmed(roots: &[PathBuf], dir: &Path, saved: &[PathBuf], stamp: u64) -> Result<bool> {
    for file in saved.iter().take(PREVIEW_LIMIT) {
        let dest = destination(roots, dir, file)?;
        output::preview(planned("restore", &dest)?, dest.display());
    }

    if saved.len() > PREVIEW_LIMIT {
        output::hint(format!("... and {} more", saved.len() - PREVIEW_LIMIT));
    }

    output::confirm(
        "restore",
        &format!("Put {} file(s) of backup {stamp} back?", saved.len()),
        YES_FLAGS,
    )
}

fn summary(action: &str, saved: usize, created: usize, stamp: u64) -> String {
    format!(
        "{action} {saved} file(s) from backup {stamp} ({created} created, {} replaced)",
        saved.saturating_sub(created)
    )
}

fn chosen<'a>(taken: &'a [(u64, PathBuf)], name: Option<&String>) -> Result<(u64, &'a Path)> {
    let Some(name) = name else {
        let (stamp, dir) = taken.last().context("restore: no backup taken yet")?;
        return Ok((*stamp, dir.as_path()));
    };

    let Some((stamp, dir)) = taken.iter().find(|(stamp, _)| stamp.to_string() == *name) else {
        bail!("restore: no backup named {name}; run `luadot restore --list` to see them");
    };

    Ok((*stamp, dir.as_path()))
}

fn destination(roots: &[PathBuf], dir: &Path, file: &Path) -> Result<PathBuf> {
    let relative = file.strip_prefix(dir).with_context(|| {
        format!(
            "restore: {} cannot be placed back from the backup {}",
            file.display(),
            dir.display()
        )
    })?;

    let dest = Path::new("/").join(relative);
    if roots
        .iter()
        .any(|root| utils::managed_relative(root, &dest).is_ok())
    {
        return Ok(dest);
    }

    bail!(
        "restore: {} is outside what luadot manages ({})",
        dest.display(),
        roots
            .iter()
            .map(|root| root.display().to_string())
            .collect::<Vec<String>>()
            .join(", ")
    )
}

fn planned(command: &str, dest: &Path) -> Result<SyncOutcome> {
    match files::exists(command, dest)? {
        true => Ok(SyncOutcome::Replaced),
        false => Ok(SyncOutcome::Created),
    }
}

fn put_back(
    command: &str,
    roots: &[PathBuf],
    dir: &Path,
    file: &Path,
) -> Result<(PathBuf, SyncOutcome)> {
    let dest = destination(roots, dir, file)?;
    let outcome = planned(command, &dest)?;

    files::replace_file(command, &dest, |staged| {
        backup::copy_entry(command, file, staged)
    })?;

    Ok((dest, outcome))
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

    fn backed(dir: &Path, path: &Path) -> PathBuf {
        dir.join(path.strip_prefix("/").unwrap())
    }

    fn roots() -> Vec<PathBuf> {
        vec![PathBuf::from("/home/u"), PathBuf::from("/srv/dots")]
    }

    #[test]
    fn restores_to_the_absolute_path() {
        let dir = Path::new("/data/backups/100");

        assert_eq!(
            destination(&roots(), dir, &dir.join("home/u/.zshrc")).unwrap(),
            PathBuf::from("/home/u/.zshrc")
        );

        let err = destination(&roots(), dir, Path::new("/elsewhere/.zshrc"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("cannot be placed back from the backup"));
    }

    #[test]
    fn a_backup_reaching_past_the_home_and_the_repository_is_refused() {
        let dir = Path::new("/data/backups/100");

        assert_eq!(
            destination(&roots(), dir, &dir.join("srv/dots/.zshrc")).unwrap(),
            PathBuf::from("/srv/dots/.zshrc")
        );

        let err = destination(&roots(), dir, &dir.join("etc/cron.d/luadot"))
            .unwrap_err()
            .to_string();

        assert_eq!(
            err,
            "restore: /etc/cron.d/luadot is outside what luadot manages (/home/u, /srv/dots)"
        );
    }

    #[test]
    fn a_lost_file_comes_back_created() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("backup");
        let home = root.path().join("home");
        let saved = backed(&dir, &home.join(".zshrc"));
        std::fs::create_dir_all(saved.parent().unwrap()).unwrap();
        std::fs::write(&saved, "handwritten").unwrap();

        let (dest, outcome) = put_back("restore", &[home], &dir, &saved).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "handwritten");
    }

    #[test]
    fn restoring_spares_the_linked_file() {
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

        let (_, outcome) = put_back("restore", std::slice::from_ref(&home), &dir, &saved).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(
            std::fs::read_to_string(home.join(".zshrc")).unwrap(),
            "handwritten"
        );
        assert_eq!(std::fs::read_to_string(&managed).unwrap(), "managed");
    }
}
