use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use crate::lua::{self, DEFINITIONS};
use crate::utils;

#[derive(Debug, Args)]
pub struct MetaArgs {
    #[command(subcommand)]
    pub action: Option<MetaAction>,
}

#[derive(Debug, Subcommand)]
pub enum MetaAction {
    #[command(
        about = "Write the definitions into the data directory and a .luarc.json loading them into the configuration directory"
    )]
    Install(MetaInstallArgs),
}

#[derive(Debug, Args)]
pub struct MetaInstallArgs {
    #[arg(
        value_name = "DIR",
        help = "One directory to write the .luarc.json into, instead of the configuration directory"
    )]
    pub dir: Option<PathBuf>,
}

pub fn meta_cmd(args: MetaArgs) -> Result<()> {
    match args.action {
        None => print(),
        Some(MetaAction::Install(args)) => install(args),
    }
}

fn print() -> Result<()> {
    print!("{DEFINITIONS}");
    Ok(())
}

fn install(args: MetaInstallArgs) -> Result<()> {
    let config = lua::load_config()?;
    let registered = utils::configured("meta", &config)?.runtime_paths().to_vec();

    utils::place_definitions("meta", &root(args.dir)?, &registered)
}

fn root(dir: Option<PathBuf>) -> Result<PathBuf> {
    let Some(dir) = dir else {
        return utils::config_dir();
    };

    std::path::absolute(&dir).with_context(|| format!("meta: invalid path {}", dir.display()))
}
