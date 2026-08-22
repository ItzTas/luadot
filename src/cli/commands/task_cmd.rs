use anyhow::{Result, bail};
use clap::Args;

use super::super::constants::{TASK_NONE, TASK_UNKNOWN};
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

    run("task", &config, &args.name.unwrap_or_default(), args.args)
}

pub fn external_cmd(words: Vec<String>) -> Result<()> {
    let config = lua::load_config()?;
    let Some((name, args)) = words.split_first() else {
        bail!(TASK_NONE);
    };

    run(name, &config, name, args.to_vec())
}

fn list(config: &Shared) -> Result<()> {
    for (name, _) in utils::configured("task", config)?.tasks() {
        output::line(name);
    }

    Ok(())
}

fn run(command: &str, config: &Shared, name: &str, args: Vec<String>) -> Result<()> {
    let found = registered(command, config, name)?;
    let Some(task) = found else {
        bail!(unknown(command, config, name)?);
    };

    utils::said(task.run(&format!("task `{name}`"), args)?);

    Ok(())
}

fn registered(command: &str, config: &Shared, name: &str) -> Result<Option<Task>> {
    Ok(utils::configured(command, config)?.task(name).cloned())
}

fn unknown(command: &str, config: &Shared, name: &str) -> Result<String> {
    let names: Vec<String> = utils::configured(command, config)?
        .tasks()
        .map(|(name, _)| name.to_string())
        .collect();
    let registered = match names.is_empty() {
        true => "none".to_string(),
        false => names.join(", "),
    };

    Ok(format!(
        "`{name}` {TASK_UNKNOWN} (registered: {registered})"
    ))
}
