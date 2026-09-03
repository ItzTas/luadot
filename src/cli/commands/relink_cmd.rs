use anyhow::Result;
use clap::Args;

use crate::utils::Placer;

#[derive(Debug, Args)]
pub struct RelinkArgs {
    #[arg(value_name = "PATH", help = "Narrow the run to this file or directory")]
    pub path: Option<String>,
    #[arg(
        short = 'n',
        long,
        help = "Report what would be linked again, touching nothing and taking no backup"
    )]
    pub dry_run: bool,
}

pub fn relink_cmd(args: RelinkArgs) -> Result<()> {
    Placer::RELINK.place(args.path.as_deref(), args.dry_run)
}
