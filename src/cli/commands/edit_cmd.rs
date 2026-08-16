use anyhow::Result;
use clap::Args;

use crate::{lua, utils};

#[derive(Debug, Args)]
pub struct EditArgs {
    #[arg(value_name = "PATH")]
    pub path: String,
}

pub fn edit_cmd(args: EditArgs) -> Result<()> {
    let repo = utils::require_repo("edit", lua::load_config()?.repo_dir())?;
    let home = utils::home_dir()?;
    let in_repo = utils::managed_path("edit", &home, &repo, &args.path)?;

    utils::open("edit", &in_repo)
}
