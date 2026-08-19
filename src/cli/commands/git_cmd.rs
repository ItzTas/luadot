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
        allow_hyphen_values = true
    )]
    pub args: Vec<String>,
}

pub fn git_cmd(args: GitArgs) -> Result<()> {
    let repo = utils::require_repo("git", lua::load_config()?.repo_dir())?;

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

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use super::build_command;

    #[test]
    fn runs_git_in_repo_dir_forwarding_all_args() {
        let command = build_command(
            Path::new("/tmp/luadot-repo"),
            &["commit".to_string(), "-m".to_string(), "msg".to_string()],
        );

        assert_eq!(command.get_program(), OsStr::new("git"));
        assert_eq!(
            command.get_current_dir(),
            Some(Path::new("/tmp/luadot-repo"))
        );

        let args: Vec<&str> = command.get_args().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(args, ["commit", "-m", "msg"]);
    }
}
