use anyhow::Result;
use clap::Args;

use crate::utils::Placer;

#[derive(Debug, Args)]
pub struct ApplyArgs {
    #[arg(value_name = "PATH", help = "Narrow the run to this file or directory")]
    pub path: Option<String>,
    #[arg(
        short = 'n',
        long,
        help = "Report what would change, touching nothing and taking no backup"
    )]
    pub dry_run: bool,
}

pub fn apply_cmd(args: ApplyArgs) -> Result<()> {
    Placer::APPLY.place(args.path.as_deref(), args.dry_run)
}
