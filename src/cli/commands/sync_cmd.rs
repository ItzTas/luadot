use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::files::{self, ConflictPolicy, LinkMode, SyncOutcome};
use crate::{state, utils};

pub fn sync_cmd(args: &[String]) -> Result<()> {
    let repo = repo_dir()?;
    if !repo.is_dir() {
        bail!(
            "sync: repository {} does not exist; run `luadot clone <url>` first",
            repo.display()
        );
    }

    let home = utils::home_dir()?;

    let root = match args.first() {
        Some(arg) => {
            let target =
                std::path::absolute(arg).with_context(|| format!("sync: invalid path {arg}"))?;
            utils::repo_path(&home, &repo, &target)?
        }
        None => repo.clone(),
    };

    if !root.exists() {
        bail!("sync: {} is not managed by the repository", root.display());
    }

    let files = collect_files(&root)?;
    if files.is_empty() {
        println!("luadot: nothing to sync");
        return Ok(());
    }

    let mut created = 0u32;
    let mut replaced = 0u32;
    let mut unchanged = 0u32;
    for file in &files {
        let dest = utils::system_path(&home, &repo, file)?;
        // Hard links (with a copy fallback) are the only strategy for now; a
        // future Lua configuration will choose between hard, symbolic links and
        // copies per file.
        let outcome = files::sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, file, &dest)
            .with_context(|| format!("sync: failed to sync {}", dest.display()))?;
        match outcome {
            SyncOutcome::Created => created += 1,
            SyncOutcome::Replaced => replaced += 1,
            SyncOutcome::AlreadySynced => unchanged += 1,
            SyncOutcome::Skipped => {}
        }
    }

    println!(
        "luadot: synced {} file(s) ({created} created, {replaced} replaced, {unchanged} unchanged)",
        files.len()
    );

    Ok(())
}

/// Collects every file under `root`, recursing into directories but skipping
/// any `.git` directory so the repository's own metadata is never synced.
fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if root.is_dir() {
        collect_into(root, &mut files)?;
    } else {
        files.push(root.to_path_buf());
    }
    files.sort();
    Ok(files)
}

fn collect_into(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("sync: failed to read {}", dir.display()))?;
    for entry in entries {
        let entry =
            entry.with_context(|| format!("sync: failed to read an entry in {}", dir.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("sync: failed to inspect {}", entry.path().display()))?;
        if file_type.is_dir() {
            if entry.file_name() == ".git" {
                continue;
            }
            collect_into(&entry.path(), files)?;
        } else {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn repo_dir() -> Result<PathBuf> {
    let state = state::load()?;
    state
        .repo
        .context("sync: no repository set; run `luadot clone <url>` first")
}

#[cfg(test)]
mod tests {
    use super::collect_files;

    #[test]
    fn collect_files_returns_a_single_file_root() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("init.lua");
        std::fs::write(&file, "data").unwrap();

        assert_eq!(collect_files(&file).unwrap(), vec![file]);
    }

    #[test]
    fn collect_files_walks_directories_recursively() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".config/nvim")).unwrap();
        std::fs::write(root.join(".bashrc"), "a").unwrap();
        std::fs::write(root.join(".config/nvim/init.lua"), "b").unwrap();

        let files = collect_files(root).unwrap();

        assert_eq!(
            files,
            vec![root.join(".bashrc"), root.join(".config/nvim/init.lua")]
        );
    }

    #[test]
    fn collect_files_skips_the_git_directory() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git/objects")).unwrap();
        std::fs::write(root.join(".git/config"), "x").unwrap();
        std::fs::write(root.join(".git/objects/blob"), "y").unwrap();
        std::fs::write(root.join(".vimrc"), "z").unwrap();

        assert_eq!(collect_files(root).unwrap(), vec![root.join(".vimrc")]);
    }
}
