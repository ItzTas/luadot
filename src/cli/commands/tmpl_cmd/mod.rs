mod alt;
mod new;

use anyhow::Result;
use clap::{Args, Subcommand};

use alt::AltArgs;
use new::NewArgs;

#[derive(Debug, Args)]
pub struct TmplArgs {
    #[command(subcommand)]
    pub action: TmplAction,
}

#[derive(Debug, Subcommand)]
pub enum TmplAction {
    #[command(about = "Create an empty template next to the file it produces and manage it")]
    New(NewArgs),
    #[command(about = "Run the templates and put the files they produce on the system")]
    Alt(AltArgs),
}

impl TmplArgs {
    pub fn dry_run(&self) -> bool {
        match &self.action {
            TmplAction::New(_) => false,
            TmplAction::Alt(args) => args.dry_run,
        }
    }
}

pub fn tmpl_cmd(args: TmplArgs) -> Result<()> {
    match args.action {
        TmplAction::New(args) => new::new(args),
        TmplAction::Alt(args) => alt::alt(args),
    }
}
