use anyhow::{Context, Result};

use crate::{git, state, utils};

pub fn clone(args: &[String]) -> Result<()> {
    const REPO_DIR: &str = "repo";

    let url = args.first().context("clone: missing repository url")?;

    let dir = utils::data_dir()?.join(REPO_DIR);
    git::clone_repo(&dir, url)?;

    let mut current = state::load()?;
    current.repo = Some(dir);
    state::save(&current)?;

    Ok(())
}
