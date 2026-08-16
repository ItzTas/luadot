use anyhow::{Result, bail};
use clap::Args;

use crate::{lua, utils};

#[derive(Debug, Args)]
pub struct SetupArgs {
    #[arg(value_name = "NAME")]
    pub names: Vec<String>,
}

pub fn setup_cmd(args: SetupArgs) -> Result<()> {
    let repo = utils::require_repo("setup", lua::load_config()?.repo_dir())?;

    if args.names.is_empty() {
        let names = lua::list_setups("setup", &repo)?;
        let available = match names.is_empty() {
            true => "none".to_string(),
            false => names.join(", "),
        };
        bail!("setup: missing setup name (available: {available})");
    }

    lua::run_setups("setup", &repo, &args.names)
}
