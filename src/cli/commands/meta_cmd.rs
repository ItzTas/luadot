use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use crate::lua::{self, DEFINITIONS};
use crate::output;
use crate::state;
use crate::utils;

use super::super::constants::META_NO_REPOSITORY;

#[derive(Debug, Args)]
pub struct MetaArgs {
    #[command(subcommand)]
    pub action: Option<MetaAction>,
}

#[derive(Debug, Subcommand)]
pub enum MetaAction {
    #[command(
        about = "Write the definitions into the data directory and a .luarc.json loading them into the configuration directory and the repository"
    )]
    Install(MetaInstallArgs),
}

#[derive(Debug, Args)]
pub struct MetaInstallArgs {
    #[arg(
        value_name = "DIR",
        help = "One directory to write the .luarc.json into, instead of those two"
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
    utils::place_definitions("meta", &roots(args.dir)?)
}

fn roots(dir: Option<PathBuf>) -> Result<Vec<PathBuf>> {
    if let Some(dir) = dir {
        let dir = std::path::absolute(&dir)
            .with_context(|| format!("meta: invalid path {}", dir.display()))?;
        return Ok(vec![dir]);
    }

    let config = lua::load_config()?;
    let configured = utils::configured("meta", &config)?
        .repo_dir()
        .map(Path::to_path_buf);
    let mut roots = vec![utils::config_dir()?];

    if configured.is_none() && state::load()?.repo().is_none() {
        output::hint(META_NO_REPOSITORY);
        return Ok(roots);
    }
    roots.push(utils::require_repo("meta", configured.as_deref())?);

    Ok(roots)
}
