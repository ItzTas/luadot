use std::ffi::OsString;

use anyhow::{Context, Result, bail};
use clap::builder::Resettable;
use clap::{Args, CommandFactory};

use super::super::constants::TASK_UNKNOWN;
use super::super::types::Cli;
use crate::lua::{self, Shared, Task};
use crate::output;
use crate::utils;

#[derive(Debug, Args)]
pub struct TaskArgs {
    #[arg(
        short,
        long,
        help = "List the tasks the configuration registers, one per line"
    )]
    pub list: bool,
    #[arg(
        value_name = "NAME",
        required_unless_present = "list",
        help = "The task to run, one --list prints; `luadot <NAME>` is the same"
    )]
    pub name: Option<String>,
    #[arg(
        value_name = "ARGS",
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "What the task receives"
    )]
    pub args: Vec<String>,
}

pub fn task_cmd(args: TaskArgs) -> Result<()> {
    let config = lua::load_config()?;
    if args.list {
        return list(&config);
    }

    let name = args.name.unwrap_or_default();
    match registered("task", &config, &name)? {
        Some(task) => run(&task, &name, args.args),
        None => bail!(
            "task: `{name}` {TASK_UNKNOWN} (registered: {})",
            names("task", &config)?
        ),
    }
}

pub fn external_cmd(words: Vec<String>) -> Result<()> {
    let config = lua::load_config()?;
    let Some((name, args)) = words.split_first() else {
        bail!("task: no task named");
    };

    match registered(name, &config, name)? {
        Some(task) => run(&task, name, args.to_vec()),
        None => refused(name, &config),
    }
}

fn list(config: &Shared) -> Result<()> {
    for (name, _) in utils::configured("task", config)?.tasks() {
        output::line(name);
    }

    Ok(())
}

fn run(task: &Task, name: &str, args: Vec<String>) -> Result<()> {
    utils::said(task.run(&format!("task `{name}`"), args)?);

    Ok(())
}

fn registered(command: &str, config: &Shared, name: &str) -> Result<Option<Task>> {
    Ok(utils::configured(command, config)?.task(name).cloned())
}

fn names(command: &str, config: &Shared) -> Result<String> {
    let names: Vec<String> = utils::configured(command, config)?
        .tasks()
        .map(|(name, _)| name.to_string())
        .collect();

    Ok(match names.is_empty() {
        true => "none".to_string(),
        false => names.join(", "),
    })
}

fn refused(command: &str, config: &Shared) -> Result<()> {
    let tasks: Vec<String> = utils::configured(command, config)?
        .tasks()
        .map(|(name, _)| name.to_string())
        .collect();
    let Some(err) = refusal(&tasks, std::env::args_os()) else {
        bail!(
            "`{command}` {TASK_UNKNOWN} (registered: {})",
            names(command, config)?
        );
    };

    err.print()
        .with_context(|| format!("{command}: failed to report the unknown command"))?;
    std::process::exit(err.exit_code());
}

fn refusal(tasks: &[String], args: impl IntoIterator<Item = OsString>) -> Option<clap::Error> {
    let mut command = Cli::command()
        .allow_external_subcommands(false)
        .external_subcommand_value_parser(Resettable::Reset);
    for task in tasks {
        command = command.subcommand(clap::Command::new(task.clone()));
    }

    command.try_get_matches_from(args).err()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    #[test]
    fn a_near_name_is_suggested() {
        let tasks = ["plug".to_string()];

        let err = refusal(&tasks, argv(&["luadot", "stauts"])).unwrap();
        assert!(err.to_string().contains("'status'"), "{err}");

        let err = refusal(&tasks, argv(&["luadot", "plugg"])).unwrap();
        assert!(err.to_string().contains("'plug'"), "{err}");
    }

    #[test]
    fn a_name_that_is_a_task_is_not_refused() {
        assert!(refusal(&["plug".to_string()], argv(&["luadot", "plug"])).is_none());
    }
}
