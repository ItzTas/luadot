use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use crate::lua::{self, DEFINITIONS, Placed};
use crate::output::{self, Tone};
use crate::state;
use crate::utils;

use super::super::constants::{META_KEPT, META_MERGED, META_NO_REPOSITORY, META_WROTE};

#[derive(Debug, Args)]
pub struct MetaArgs {
    #[command(subcommand)]
    pub action: Option<MetaAction>,
}

#[derive(Debug, Subcommand)]
pub enum MetaAction {
    #[command(
        about = "Write the definitions and a .luarc.json into the configuration directory and the repository"
    )]
    Install(MetaInstallArgs),
}

#[derive(Debug, Args)]
pub struct MetaInstallArgs {
    #[arg(
        value_name = "DIR",
        help = "One directory to write into, instead of those two"
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
    for dir in roots(args.dir)? {
        for placed in lua::install_definitions("meta", &dir)? {
            report(&placed);
        }
    }

    Ok(())
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

fn report(placed: &Placed) {
    match placed {
        Placed::Written(path) => output::entry(Tone::Good, META_WROTE, path.display()),
        Placed::Merged(path) => output::entry(Tone::Good, META_MERGED, path.display()),
        Placed::Kept(path, wanted) => {
            output::warn(format!("meta: {} {META_KEPT}", path.display()));
            output::line(wanted);
        }
    }
}
