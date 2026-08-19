use std::path::Path;

use anyhow::{Result, bail};
use clap::Args;

use crate::output;
use crate::{lua, utils};

#[derive(Debug, Args)]
pub struct SetupArgs {
    #[arg(
        short,
        long,
        help = "List the setups the repository declares, one per line"
    )]
    pub list: bool,
    #[arg(value_name = "NAME")]
    pub names: Vec<String>,
}

pub fn setup_cmd(args: SetupArgs) -> Result<()> {
    let repo = utils::require_repo("setup", lua::load_config()?.repo_dir())?;

    if args.list {
        return list(&repo);
    }

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

fn list(repo: &Path) -> Result<()> {
    for name in lua::list_setups("setup", repo)? {
        output::line(name);
    }
    Ok(())
}
