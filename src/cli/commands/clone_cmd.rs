use std::path::Path;

use anyhow::Result;
use clap::Args;

use crate::{git, lua, state, utils};

#[derive(Debug, Args)]
pub struct CloneArgs {
    #[arg(value_name = "URL")]
    pub url: String,
}

pub fn clone_cmd(args: CloneArgs) -> Result<()> {
    const REPO_DIR: &str = "repo";

    let dir = utils::data_dir()?.join(REPO_DIR);
    git::clone(&dir, &args.url)?;

    let mut current = state::load()?;
    current.set_repo(dir.clone());
    state::save(&current)?;

    offer_bootstrap(&dir)
}

fn offer_bootstrap(repo: &Path) -> Result<()> {
    let path = lua::bootstrap_path("clone", repo)?;
    if !path.is_file() {
        return Ok(());
    }

    let question = format!("clone: found {}. Run it now?", path.display());
    if !utils::offer("clone", &question)? {
        return Ok(());
    }

    utils::ask_missing("clone", lua::load_config()?.classes())?;

    lua::run_bootstrap("clone", repo)
}
