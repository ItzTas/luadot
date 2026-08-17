use std::path::{Path, PathBuf};

use anyhow::Result;

use super::paths::{home_dir, is_managed, relative};
use super::repo::{managed_path, require_repo};
use crate::files::{self, Entry};
use crate::lua::{self, Config};

pub struct Workspace {
    pub config: Config,
    pub home: PathBuf,
    pub repo: PathBuf,
}

pub fn workspace(command: &str) -> Result<Workspace> {
    let config = lua::load_config()?;
    let repo = require_repo(command, config.repo_dir())?;
    let home = home_dir()?;

    Ok(Workspace { config, home, repo })
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
