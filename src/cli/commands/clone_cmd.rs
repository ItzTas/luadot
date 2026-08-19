use std::path::Path;

use anyhow::Result;
use clap::Args;

use crate::{git, lua, output, state, utils};

#[derive(Debug, Args)]
pub struct CloneArgs {
    #[arg(value_name = "URL")]
    pub url: String,
    #[arg(value_name = "DIR")]
    pub dir: Option<String>,
}

pub fn clone_cmd(args: CloneArgs) -> Result<()> {
    let home = utils::home_dir()?;
    let configured = lua::load_config()?.repo_dir().map(Path::to_path_buf);

    let dir = utils::destination("clone", &home, args.dir.as_deref(), configured.as_deref())?;
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
    if !output::offer("clone", &question)? {
        return Ok(());
    }

    utils::ask_missing("clone", lua::load_config()?.classes())?;

    lua::run_bootstrap("clone", repo)
}
