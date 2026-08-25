use anyhow::Result;
use clap::Args;

use super::{GitArgs, git_cmd};

#[derive(Debug, Args)]
#[command(disable_help_flag = true)]
pub struct PushArgs {
    #[arg(
        value_name = "ARGS",
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "The arguments git push receives, verbatim"
    )]
    pub args: Vec<String>,
}

pub fn push_cmd(args: PushArgs) -> Result<()> {
    git_cmd(GitArgs {
        args: push_args(args.args),
    })
}

fn push_args(args: Vec<String>) -> Vec<String> {
    let mut forwarded = vec!["push".to_string()];
    forwarded.extend(args);
    forwarded
}
