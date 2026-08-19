use std::io::IsTerminal;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use super::commands;
use super::constants::{DEFAULT_FILTER, TRACE_FILTER, VERBOSE_FILTER};
use super::types::{Cli, Cmd};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    dispatch(cli)
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
    match cli.command {
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
        Cmd::Edit(args) => commands::edit_cmd(args),
        Cmd::Exec(args) => commands::exec_cmd(args),
        Cmd::Git(args) => commands::git_cmd(args),
        Cmd::Init(args) => commands::init_cmd(args),
        Cmd::Push(args) => commands::push_cmd(args),
        Cmd::Rekey(args) => commands::rekey_cmd(args),
        Cmd::Rm(args) => commands::rm_cmd(args),
        Cmd::Setup(args) => commands::setup_cmd(args),
        Cmd::Status(args) => commands::status_cmd(args),
        Cmd::Sync(args) => commands::sync_cmd(args),
        Cmd::Tmpl(args) => commands::tmpl_cmd(args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbosity_picks_the_filter() {
        assert_eq!(filter(1).to_string(), VERBOSE_FILTER);
        assert_eq!(filter(2).to_string(), TRACE_FILTER);
        assert_eq!(filter(9).to_string(), TRACE_FILTER);
    }
}
