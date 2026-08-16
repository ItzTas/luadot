use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::{git, lua, state, utils};

const REPO_DIR: &str = "repo";

#[derive(Debug, Args)]
pub struct CloneArgs {
    #[arg(value_name = "URL")]
    pub url: String,
    #[arg(value_name = "DIR")]
    pub dir: Option<String>,
}

pub fn clone_cmd(args: CloneArgs) -> Result<()> {
    let home = utils::home_dir()?;
    let configured = lua::load_config()?.repo_dir().map(Path::to_path_buf);

    let dir = destination(&home, args.dir.as_deref(), configured.as_deref())?;
    git::clone(&dir, &args.url)?;

    let mut current = state::load()?;
    current.set_repo(dir.clone());
    state::save(&current)?;

    offer_bootstrap(&dir)
}

fn destination(home: &Path, arg: Option<&str>, configured: Option<&Path>) -> Result<PathBuf> {
    if let Some(arg) = arg {
        return std::path::absolute(arg).with_context(|| format!("clone: invalid path {arg}"));
    }
    if let Some(configured) = configured {
        return Ok(utils::expand(home, configured));
    }

    Ok(utils::data_dir()?.join(REPO_DIR))
}

fn offer_bootstrap(repo: &Path) -> Result<()> {
    let path = lua::bootstrap_path("clone", repo)?;
    if !path.is_file() {
        return Ok(());
    }

    let question = format!("clone: found {}. Run it now?", path.display());
    if !utils::offer("clone", &question)? {
        return Ok(());
    }

    utils::ask_missing("clone", lua::load_config()?.classes())?;

    lua::run_bootstrap("clone", repo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_argument_wins_over_everything_else() {
        let dir = destination(
            Path::new("/home/u"),
            Some("/data/dotfiles"),
            Some(Path::new("~/configured")),
        )
        .unwrap();

        assert_eq!(dir, PathBuf::from("/data/dotfiles"));
    }

    #[test]
    fn the_configured_directory_is_used_when_no_argument_is_given() {
        let dir = destination(Path::new("/home/u"), None, Some(Path::new("~/dotfiles"))).unwrap();

        assert_eq!(dir, PathBuf::from("/home/u/dotfiles"));
    }

    #[test]
    fn without_either_it_lands_where_luadot_keeps_its_data() {
        let dir = destination(Path::new("/home/u"), None, None).unwrap();

        assert_eq!(dir, utils::data_dir().unwrap().join(REPO_DIR));
    }
}
