use std::fs::Metadata;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::files;
use crate::lua;
use crate::output;
use crate::utils;

const PREVIEW_LIMIT: usize = 10;

const YES_FLAGS: &str = "-y or --yes";

#[derive(Debug, Args)]
pub struct RmArgs {
    #[arg(value_name = "PATH", required = true)]
    pub paths: Vec<String>,
    #[arg(short, long, help = "Stop managing them without asking first")]
    pub yes: bool,
    #[arg(short = 'n', long, help = "Report what would happen, touching nothing")]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Detached {
    Restored,
    Untouched,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Plan {
    Keep,
    Copy,
    Relink,
}

pub fn rm_cmd(args: RmArgs) -> Result<()> {
    let config = lua::load_config()?;
    let repo = utils::require_repo("rm", config.repo_dir())?;
    let home = utils::home_dir()?;

    let files = plan(&home, &repo, &args.paths)?;
    if files.is_empty() {
        output::note("nothing to remove");
        return Ok(());
    }

    if args.dry_run {
        return foresee(&home, &repo, &files);
    }

    if !args.yes && !confirmed(&repo, &files)? {
        output::warn("aborted");
        return Ok(());
    }

    let mut restored = 0u32;
    let mut untouched = 0u32;
    for file in &files {
        let dest = utils::system_path(&home, &repo, file)?;
        match detach(file, &dest)? {
            Detached::Restored => restored += 1,
            Detached::Untouched => untouched += 1,
        }
        std::fs::remove_file(file)
            .with_context(|| format!("rm: failed to remove {}", file.display()))?;
        prune_parents(&repo, file)?;
    }

    output::note(format!(
        "stopped managing {} file(s) ({restored} restored, {untouched} left untouched)",
        files.len()
    ));

    Ok(())
}

fn foresee(home: &Path, repo: &Path, files: &[PathBuf]) -> Result<()> {
    output::line(preview(repo, files, files.len()));

    let mut restored = 0u32;
    let mut untouched = 0u32;
    for file in files {
        let dest = utils::system_path(home, repo, file)?;
        match decide(file, &dest)?.detached() {
            Detached::Restored => restored += 1,
            Detached::Untouched => untouched += 1,
        }
    }

    output::note(format!(
        "would stop managing {} file(s) ({restored} restored, {untouched} left untouched)",
        files.len()
    ));

    Ok(())
}

fn confirmed(repo: &Path, files: &[PathBuf]) -> Result<bool> {
    if files.len() < 2 {
        return Ok(true);
    }

    output::line(preview(repo, files, PREVIEW_LIMIT));
    utils::confirm(
        "rm",
        &format!("Stop managing {} file(s)?", files.len()),
        YES_FLAGS,
    )
}

fn preview(repo: &Path, files: &[PathBuf], limit: usize) -> String {
    let mut lines: Vec<String> = files
        .iter()
        .take(limit)
        .map(|file| format!("  {}", utils::relative(repo, file).display()))
        .collect();

    if files.len() > limit {
        lines.push(format!("  ... and {} more", files.len() - limit));
    }

    lines.join("\n")
}

fn plan(home: &Path, repo: &Path, args: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for arg in args {
        let root = utils::managed_path("rm", home, repo, arg)?;
        files.extend(files::collect_files("rm", &root)?);
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn detach(source: &Path, dest: &Path) -> Result<Detached> {
    let plan = decide(source, dest)?;

    if plan == Plan::Relink {
        std::fs::remove_file(dest)
            .with_context(|| format!("rm: failed to remove {}", dest.display()))?;
    }
    if plan != Plan::Keep {
        restore(source, dest)?;
    }

    Ok(plan.detached())
}

fn decide(source: &Path, dest: &Path) -> Result<Plan> {
    if metadata(source)?.is_some_and(|meta| meta.file_type().is_symlink()) {
        return Ok(Plan::Keep);
    }

    let Some(meta) = metadata(dest)? else {
        return Ok(Plan::Copy);
    };

    if !meta.file_type().is_symlink() || !points_at(dest, source)? {
        return Ok(Plan::Keep);
    }

    Ok(Plan::Relink)
}

impl Plan {
    fn detached(self) -> Detached {
        match self {
            Self::Keep => Detached::Untouched,
            Self::Copy | Self::Relink => Detached::Restored,
        }
    }
}

fn restore(source: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("rm: failed to create {}", parent.display()))?;
    }

    std::fs::copy(source, dest).map(|_| ()).with_context(|| {
        format!(
            "rm: failed to restore {} from {}",
            dest.display(),
            source.display()
        )
    })
}

fn prune_parents(repo: &Path, file: &Path) -> Result<()> {
    let mut current = file.parent();
    while let Some(dir) = current.filter(|dir| *dir != repo && dir.starts_with(repo)) {
        if !is_empty(dir)? {
            return Ok(());
        }
        std::fs::remove_dir(dir)
            .with_context(|| format!("rm: failed to remove {}", dir.display()))?;
        current = dir.parent();
    }
    Ok(())
}

fn is_empty(dir: &Path) -> Result<bool> {
    let mut entries =
        std::fs::read_dir(dir).with_context(|| format!("rm: failed to read {}", dir.display()))?;
    Ok(entries.next().is_none())
}

fn metadata(path: &Path) -> Result<Option<Metadata>> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) => Ok(Some(meta)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("rm: failed to inspect {}", path.display())),
    }
}

fn points_at(link: &Path, target: &Path) -> Result<bool> {
    let read = std::fs::read_link(link)
        .with_context(|| format!("rm: failed to read {}", link.display()))?;
    Ok(read == target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, contents: &str) {
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn a_single_file_needs_no_confirmation() {
        let repo = Path::new("/repo");

        assert!(confirmed(repo, &[repo.join(".bashrc")]).unwrap());
    }

    #[test]
    fn preview_lists_the_files_relative_to_the_repository() {
        let repo = Path::new("/repo");
        let files = vec![repo.join(".bashrc"), repo.join(".config/nvim/init.lua")];

        assert_eq!(
            preview(repo, &files, PREVIEW_LIMIT),
            "  .bashrc\n  .config/nvim/init.lua"
        );
    }

    #[test]
    fn preview_truncates_a_long_list() {
        let repo = Path::new("/repo");
        let files: Vec<PathBuf> = (0..PREVIEW_LIMIT + 3)
            .map(|index| repo.join(format!(".file{index}")))
            .collect();

        let preview = preview(repo, &files, PREVIEW_LIMIT);

        assert_eq!(preview.lines().count(), PREVIEW_LIMIT + 1);
        assert!(preview.ends_with("  ... and 3 more"));
    }

    #[test]
    fn a_plan_says_what_detaching_would_do_without_doing_it() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");

        assert_eq!(decide(&source, &dest).unwrap(), Plan::Copy);
        assert!(!dest.exists());

        std::os::unix::fs::symlink(&source, &dest).unwrap();
        assert_eq!(decide(&source, &dest).unwrap(), Plan::Relink);
        assert!(std::fs::symlink_metadata(&dest).unwrap().is_symlink());
    }

    #[test]
    fn every_plan_says_how_the_file_ends_up() {
        assert_eq!(Plan::Keep.detached(), Detached::Untouched);
        assert_eq!(Plan::Copy.detached(), Detached::Restored);
        assert_eq!(Plan::Relink.detached(), Detached::Restored);
    }

    #[test]
    fn detach_restores_a_missing_system_file() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("nested").join("dest");
        write(&source, "data");

        assert_eq!(detach(&source, &dest).unwrap(), Detached::Restored);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
    }

    #[test]
    fn detach_materializes_a_symlink_pointing_into_the_repository() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        std::os::unix::fs::symlink(&source, &dest).unwrap();

        assert_eq!(detach(&source, &dest).unwrap(), Detached::Restored);

        let kind = std::fs::symlink_metadata(&dest).unwrap().file_type();
        assert!(!kind.is_symlink());
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
    }

    #[test]
    fn detach_leaves_a_hard_linked_system_file_alone() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        std::fs::hard_link(&source, &dest).unwrap();

        assert_eq!(detach(&source, &dest).unwrap(), Detached::Untouched);

        std::fs::remove_file(&source).unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
    }

    #[test]
    fn detach_keeps_a_diverging_system_file() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "repo");
        write(&dest, "system");

        assert_eq!(detach(&source, &dest).unwrap(), Detached::Untouched);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "system");
    }

    #[test]
    fn detach_keeps_a_symlink_pointing_somewhere_else() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        let other = dir.path().join("other");
        write(&source, "repo");
        write(&other, "other");
        std::os::unix::fs::symlink(&other, &dest).unwrap();

        assert_eq!(detach(&source, &dest).unwrap(), Detached::Untouched);
        assert!(
            std::fs::symlink_metadata(&dest)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn detach_leaves_the_system_file_when_the_repository_holds_a_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&dest, "system");
        std::os::unix::fs::symlink(&dest, &source).unwrap();

        assert_eq!(detach(&source, &dest).unwrap(), Detached::Untouched);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "system");
    }

    #[test]
    fn prune_parents_removes_empty_directories_up_to_the_repository() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        let nested = repo.join(".config").join("nvim");
        std::fs::create_dir_all(&nested).unwrap();

        prune_parents(&repo, &nested.join("init.lua")).unwrap();

        assert!(!repo.join(".config").exists());
        assert!(repo.is_dir());
    }

    #[test]
    fn prune_parents_stops_at_a_directory_that_still_holds_files() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        let nested = repo.join(".config").join("nvim");
        std::fs::create_dir_all(&nested).unwrap();
        write(&repo.join(".config").join("keep"), "keep");

        prune_parents(&repo, &nested.join("init.lua")).unwrap();

        assert!(!nested.exists());
        assert!(repo.join(".config").is_dir());
    }

    #[test]
    fn plan_collects_every_file_below_a_managed_directory() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = repo.join(".config").join("nvim");
        std::fs::create_dir_all(nvim.join("lua")).unwrap();
        write(&nvim.join("init.lua"), "init");
        write(&nvim.join("lua").join("plugins.lua"), "plugins");

        let arg = home
            .join(".config")
            .join("nvim")
            .to_string_lossy()
            .into_owned();
        let files = plan(&home, &repo, &[arg]).unwrap();

        assert_eq!(
            files,
            vec![nvim.join("init.lua"), nvim.join("lua").join("plugins.lua")]
        );
    }

    #[test]
    fn plan_deduplicates_repeated_arguments() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        write(&repo.join(".bashrc"), "data");

        let arg = home.join(".bashrc").to_string_lossy().into_owned();
        let files = plan(&home, &repo, &[arg.clone(), arg]).unwrap();

        assert_eq!(files, vec![repo.join(".bashrc")]);
    }
}
