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

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_handler(_args: &[String]) -> Result<()> {
        Ok(())
    }

    fn echo_handler(args: &[String]) -> Result<()> {
        anyhow::bail!("args=[{}]", args.join(","))
    }

    fn commands_from(entries: Vec<(&'static str, Command)>) -> HashMap<&'static str, Command> {
        entries.into_iter().collect()
    }

    #[test]
    fn runs_matching_command_with_rest_args() {
        let commands = commands_from(vec![("echo", Command::Run(echo_handler))]);
        let args = vec!["echo".to_string(), "a".to_string(), "b".to_string()];

        let err = dispatch(&commands, &args).unwrap_err();
        assert_eq!(err.to_string(), "args=[a,b]");
    }

    #[test]
    fn descends_into_groups() {
        let inner = commands_from(vec![("sub", Command::Run(echo_handler))]);
        let commands = commands_from(vec![("group", Command::Group(inner))]);
        let args = vec!["group".to_string(), "sub".to_string(), "x".to_string()];

        let err = dispatch(&commands, &args).unwrap_err();
        assert_eq!(err.to_string(), "args=[x]");
    }

    #[test]
    fn unknown_command_errors_and_lists_available() {
        let commands = commands_from(vec![("clone", Command::Run(ok_handler))]);
        let args = vec!["nope".to_string()];

        let err = dispatch(&commands, &args).unwrap_err().to_string();
        assert!(err.contains("unknown command: nope"));
        assert!(err.contains("available: clone"));
    }

    #[test]
    fn no_command_errors() {
        let commands = commands_from(vec![]);
        let err = dispatch(&commands, &[]).unwrap_err().to_string();
        assert!(err.contains("no command given"));
    }

    #[test]
    fn available_is_sorted_and_joined() {
        let commands = commands_from(vec![
            ("zeta", Command::Run(ok_handler)),
            ("alpha", Command::Run(ok_handler)),
            ("mike", Command::Run(ok_handler)),
        ]);
        assert_eq!(available(&commands), "alpha, mike, zeta");
    }
}
