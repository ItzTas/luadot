mod cli;
mod files;
mod git;
mod lua;
mod state;
mod utils;

use std::collections::HashMap;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};

use cli::Command;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("luadot: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    dispatch(&cli::get_commands(), &args)
}

fn dispatch(commands: &HashMap<&'static str, Command>, args: &[String]) -> Result<()> {
    let (name, rest) = args
        .split_first()
        .with_context(|| format!("no command given (available: {})", available(commands)))?;

    match commands.get(name.as_str()) {
        Some(Command::Run(handler)) => handler(rest),
        Some(Command::Group(sub)) => dispatch(sub, rest),
        None => bail!("unknown command: {name} (available: {})", available(commands)),
    }
}

fn available(commands: &HashMap<&'static str, Command>) -> String {
    let mut names: Vec<&str> = commands.keys().copied().collect();
    names.sort_unstable();
    names.join(", ")
}
