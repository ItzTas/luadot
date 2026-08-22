use std::path::Path;

use anyhow::Result;
use clap::Args;

use crate::output::Tone;
use crate::{git, lua, output, state, utils};

use super::super::constants::CONFIG_WROTE;

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
    let (configured, lfs, registered) = {
        let loaded = utils::configured("init", &config)?;
        (
            loaded.repo_dir().map(Path::to_path_buf),
            loaded.lfs(),
            loaded.runtime_paths().to_vec(),
        )
    };

    let dir = utils::destination("init", &home, args.dir.as_deref(), configured.as_deref())?;
    git::init(&dir, lfs)?;

    let mut current = state::load()?;
    current.set_repo(dir.clone());
    state::save(&current)?;

    output::note(format!("created {}", dir.display()));
    if let Some(path) = lua::place_starter("init", &lua::config_path()?)? {
        output::entry(Tone::Good, CONFIG_WROTE, path.display());
    }
    utils::offer_definitions("init", &registered);

    Ok(())
}
