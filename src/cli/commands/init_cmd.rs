use std::path::Path;

use anyhow::Result;
use clap::Args;

use crate::{git, lua, output, state, utils};

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(
        value_name = "DIR",
        help = "Where the repository is created, the default place when left out"
    )]
    pub dir: Option<String>,
}

pub fn init_cmd(args: InitArgs) -> Result<()> {
    let home = utils::home_dir()?;
    let config = lua::load_config()?;
    let configured = utils::configured("init", &config)?
        .repo_dir()
        .map(Path::to_path_buf);

    let dir = utils::destination("init", &home, args.dir.as_deref(), configured.as_deref())?;
    git::init(&dir)?;

    let mut current = state::load()?;
    current.set_repo(dir.clone());
    state::save(&current)?;

    output::note(format!("created {}", dir.display()));

    Ok(())
}
