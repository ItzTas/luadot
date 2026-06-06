use std::collections::HashMap;

use anyhow::{Context, Result, bail};

use super::{Command, get_commands};

pub fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    dispatch(&get_commands(), &args)
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
