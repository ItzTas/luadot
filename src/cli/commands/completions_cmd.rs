use anyhow::{Context, Result};
use clap::{Args, CommandFactory};
use clap_complete::Shell;

use crate::cli::Cli;

use super::super::completions::{
    BASH_GIT_COMPLETION, FISH_GIT_COMPLETION, ZSH_DISPATCH, ZSH_GIT_COMPLETION,
};

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    #[arg(value_name = "SHELL", value_enum, help = "The shell the script is for")]
    pub shell: Shell,
}

pub fn completions_cmd(args: CompletionsArgs) -> Result<()> {
    print!("{}", script(args.shell)?);
    Ok(())
}

fn script(shell: Shell) -> Result<String> {
    let generated = generated(shell)?;

    match shell {
        Shell::Bash => Ok(format!("{generated}\n{BASH_GIT_COMPLETION}")),
        Shell::Fish => Ok(format!("{generated}\n{FISH_GIT_COMPLETION}")),
        Shell::Zsh => zsh(&generated),
        _ => Ok(generated),
    }
}

fn generated(shell: Shell) -> Result<String> {
    let mut command = Cli::command();
    let name = command.get_name().to_string();

    let mut script = Vec::new();
    clap_complete::generate(shell, &mut command, name, &mut script);

    String::from_utf8(script).context("completions: the generated script is not valid UTF-8")
}

fn zsh(generated: &str) -> Result<String> {
    let (body, _) = generated
        .rsplit_once(ZSH_DISPATCH)
        .context("completions: the generated zsh script has no dispatch to replace")?;

    Ok(format!("{body}{ZSH_GIT_COMPLETION}"))
}
