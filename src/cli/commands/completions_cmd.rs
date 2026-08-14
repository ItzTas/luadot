use anyhow::Result;
use clap::{Args, CommandFactory};
use clap_complete::Shell;

use crate::cli::Cli;

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    #[arg(value_name = "SHELL", value_enum)]
    pub shell: Shell,
}

pub fn completions_cmd(args: CompletionsArgs) -> Result<()> {
    clap_complete::generate(
        args.shell,
        &mut Cli::command(),
        "luadot",
        &mut std::io::stdout(),
    );
    Ok(())
}
