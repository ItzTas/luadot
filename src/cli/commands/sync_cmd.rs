use std::path::Path;

use anyhow::{Result, bail};
use clap::Args;

use crate::output;
use crate::utils;
use crate::{git, lua};

#[derive(Debug, Args)]
pub struct SyncArgs {
    #[arg(
        short,
        long,
        value_name = "MESSAGE",
        help = "Commit with this message instead of the default one"
    )]
    pub message: Option<String>,
    #[arg(long, help = "Commit what changed without pushing it")]
    pub no_push: bool,
}

pub fn sync_cmd(args: SyncArgs) -> Result<()> {
    let repo = utils::require_repo("sync", lua::load_config()?.repo_dir())?;
    require_git(&repo)?;

    git::stage_all("sync", &repo)?;
    if !git::commit("sync", &repo, &git::message(args.message))? {
        output::note("nothing to commit");
    }
    if args.no_push {
        return Ok(());
    }

    git::push("sync", &repo)
}

fn require_git(repo: &Path) -> Result<()> {
    if git::present(repo) {
        return Ok(());
    }

    bail!(
        "sync: {} is not a git repository; run `luadot init` or `luadot clone <url>` first",
        repo.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_without_git_metadata_says_how_to_get_one() {
        let dir = tempfile::tempdir().unwrap();

        let err = require_git(dir.path()).unwrap_err().to_string();

        assert!(err.contains("sync: "));
        assert!(err.contains("is not a git repository"));
        assert!(err.contains("luadot clone <url>"));
    }
}
