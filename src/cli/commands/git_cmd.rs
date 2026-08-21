use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use clap::Args;

use crate::{lua, utils};

#[derive(Debug, Args)]
#[command(disable_help_flag = true)]
pub struct GitArgs {
    #[arg(
        value_name = "ARGS",
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "The arguments git receives, verbatim"
    )]
    pub args: Vec<String>,
}

pub fn git_cmd(args: GitArgs) -> Result<()> {
    let config = lua::load_config()?;
    let repo = utils::require_repo("git", utils::configured("git", &config)?.repo_dir())?;

    let status = build_command(&repo, &args.args)
        .status()
        .context("git: failed to run git; is it installed and on PATH?")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn build_command(repo: &Path, args: &[String]) -> Command {
    let mut command = Command::new("git");
    command.current_dir(repo);
    command.args(args);
    command
}
