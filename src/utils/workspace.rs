use std::path::{Path, PathBuf};
use std::sync::MutexGuard;

use anyhow::{Result, anyhow};

use super::paths::{home_dir, is_managed, relative};
use super::repo::{managed_path, require_repo};
use crate::files::{self, Entry};
use crate::lua::{self, Config, Shared};

pub struct Workspace {
    pub config: Shared,
    pub home: PathBuf,
    pub repo: PathBuf,
}

pub fn workspace(command: &str) -> Result<Workspace> {
    let config = lua::load_config()?;
    let repo = require_repo(command, configured(command, &config)?.repo_dir())?;
    let home = home_dir()?;

    Ok(Workspace { config, home, repo })
}

pub fn configured<'a>(command: &str, config: &'a Shared) -> Result<MutexGuard<'a, Config>> {
    config
        .try_lock()
        .map_err(|_| anyhow!("{command}: the configuration is still being read"))
}

pub fn managed_root(
    command: &str,
    home: &Path,
    repo: &Path,
    path: Option<&str>,
) -> Result<PathBuf> {
    let Some(path) = path else {
        return Ok(repo.to_path_buf());
    };

    managed_path(command, home, repo, path)
}

pub fn managed_files(
    command: &str,
    repo: &Path,
    root: &Path,
    ignored: impl Fn(&Path) -> bool,
) -> Result<Vec<PathBuf>> {
    Ok(managed_entries(command, repo, root, ignored)?
        .into_iter()
        .filter_map(|entry| match entry {
            Entry::File(file) => Some(file),
            Entry::Template(_) | Entry::Standalone(_) => None,
        })
        .collect())
}

pub fn managed_entries(
    command: &str,
    repo: &Path,
    root: &Path,
    ignored: impl Fn(&Path) -> bool,
) -> Result<Vec<Entry>> {
    Ok(files::collect_entries(command, root)?
        .into_iter()
        .filter(|entry| {
            let target = entry.target();
            let relative = relative(repo, &target);

            is_managed(relative) && !ignored(relative)
        })
        .collect())
}
