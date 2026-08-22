use std::io::IsTerminal;
use std::thread::JoinHandle;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use super::commands;
use super::constants::{
    DEFAULT_FILTER, GUARD_FAILED, REFRESH_PANICKED, TRACE_FILTER, VERBOSE_FILTER,
};
use super::types::{Cli, Cmd};
use crate::git;
use crate::lua::Command;
use crate::output;
use crate::utils;

pub fn run() -> Result<()> {
    if let Err(err) = git::guard_locks() {
        output::warn(format!("{GUARD_FAILED}: {err}"));
    }

    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let refresh = spawn_refresh(&cli.command);
    let result = dispatch(cli);
    settle(refresh);

    result
}

fn spawn_refresh(command: &Cmd) -> Option<JoinHandle<Result<()>>> {
    if writes_definitions(command) {
        return None;
    }

    Some(std::thread::spawn(utils::refresh_definitions))
}

fn writes_definitions(command: &Cmd) -> bool {
    matches!(command, Cmd::Meta(_) | Cmd::Init(_) | Cmd::Clone(_))
}

fn settle(refresh: Option<JoinHandle<Result<()>>>) {
    let Some(handle) = refresh else {
        return;
    };
    let Ok(outcome) = handle.join() else {
        return output::warn(REFRESH_PANICKED);
    };
    if let Err(err) = outcome {
        output::warn(format!("{err:#}"));
    }
}

fn init_tracing(verbose: u8) {
    tracing_subscriber::fmt()
        .with_env_filter(filter(verbose))
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .without_time()
        .init();
}

fn filter(verbose: u8) -> EnvFilter {
    match verbose {
        0 => EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER)),
        1 => EnvFilter::new(VERBOSE_FILTER),
        _ => EnvFilter::new(TRACE_FILTER),
    }
}

fn dispatch(cli: Cli) -> Result<()> {
    utils::set_dry_run(dry_run(&cli.command));
    if let Some(command) = customized(&cli.command) {
        utils::set_command(command);
    }

    run_command(cli.command)?;

    utils::finished()
}

fn run_command(command: Cmd) -> Result<()> {
    match command {
        Cmd::Add(args) => commands::add_cmd(args),
        Cmd::Restore(args) => commands::restore_cmd(args),
        Cmd::Apply(args) => commands::apply_cmd(args),
        Cmd::Bootstrap => commands::bootstrap_cmd(),
        Cmd::Cd => commands::cd_cmd(),
        Cmd::Class(args) => commands::class_cmd(args),
        Cmd::Clone(args) => commands::clone_cmd(args),
        Cmd::Completions(args) => commands::completions_cmd(args),
        Cmd::Config(args) => commands::config_cmd(args),
        Cmd::Diff(args) => commands::diff_cmd(args),
        Cmd::Doc(args) => commands::doc_cmd(args),
        Cmd::Edit(args) => commands::edit_cmd(args),
        Cmd::Exec(args) => commands::exec_cmd(args),
        Cmd::Git(args) => commands::git_cmd(args),
        Cmd::Init(args) => commands::init_cmd(args),
        Cmd::Man => commands::man_cmd(),
        Cmd::Meta(args) => commands::meta_cmd(args),
        Cmd::Push(args) => commands::push_cmd(args),
        Cmd::Rekey(args) => commands::rekey_cmd(args),
        Cmd::Rm(args) => commands::rm_cmd(args),
        Cmd::Setup(args) => commands::setup_cmd(args),
        Cmd::Status(args) => commands::status_cmd(args),
        Cmd::Sync(args) => commands::sync_cmd(args),
        Cmd::Task(args) => commands::task_cmd(args),
        Cmd::Tmpl(args) => commands::tmpl_cmd(args),
        Cmd::External(words) => commands::external_cmd(words),
    }
}

fn dry_run(command: &Cmd) -> bool {
    match command {
        Cmd::Apply(args) => args.dry_run,
        Cmd::Rekey(args) => args.dry_run,
        Cmd::Restore(args) => args.dry_run,
        Cmd::Rm(args) => args.dry_run,
        Cmd::Tmpl(args) => args.dry_run(),
        _ => false,
    }
}

fn customized(command: &Cmd) -> Option<Command> {
    match command {
        Cmd::Add(_) => Some(Command::Add),
        Cmd::Apply(_) => Some(Command::Apply),
        Cmd::Bootstrap => Some(Command::Bootstrap),
        Cmd::Cd => Some(Command::Cd),
        Cmd::Class(_) => Some(Command::Class),
        Cmd::Clone(_) => Some(Command::Clone),
        Cmd::Config(_) => Some(Command::Config),
        Cmd::Diff(_) => Some(Command::Diff),
        Cmd::Edit(_) => Some(Command::Edit),
        Cmd::Exec(_) => Some(Command::Exec),
        Cmd::Git(_) => Some(Command::Git),
        Cmd::Init(_) => Some(Command::Init),
        Cmd::Push(_) => Some(Command::Push),
        Cmd::Rekey(_) => Some(Command::Rekey),
        Cmd::Restore(_) => Some(Command::Restore),
        Cmd::Rm(_) => Some(Command::Rm),
        Cmd::Setup(_) => Some(Command::Setup),
        Cmd::Status(_) => Some(Command::Status),
        Cmd::Sync(_) => Some(Command::Sync),
        Cmd::Tmpl(args) => Some(args.command()),
        Cmd::Completions(_) | Cmd::Doc(_) | Cmd::Man | Cmd::Meta(_) => None,
        Cmd::Task(_) | Cmd::External(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    fn parsed(args: &[&str]) -> Cmd {
        Cli::try_parse_from(args).unwrap().command
    }

    #[test]
    fn a_tmpl_action_is_customized_on_its_own() {
        assert_eq!(
            customized(&parsed(&["luadot", "tmpl", "alt"])),
            Some(Command::TmplAlt)
        );
        assert_eq!(
            customized(&parsed(&["luadot", "tmpl", "new", "~/.zshrc"])),
            Some(Command::TmplNew)
        );
    }
}
