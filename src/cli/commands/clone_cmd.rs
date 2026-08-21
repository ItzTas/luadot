use std::path::Path;

use anyhow::Result;
use clap::Args;

use crate::{git, lua, output, state, utils};

#[derive(Debug, Args)]
pub struct CloneArgs {
    #[arg(value_name = "URL", help = "The repository to clone")]
    pub url: String,
    #[arg(
        value_name = "DIR",
        help = "Where the clone lands, the default place when left out"
    )]
    pub dir: Option<String>,
}

pub fn clone_cmd(args: CloneArgs) -> Result<()> {
    let home = utils::home_dir()?;
    let config = lua::load_config()?;
    let (configured, lfs) = {
        let loaded = utils::configured("clone", &config)?;
        (loaded.repo_dir().map(Path::to_path_buf), loaded.lfs())
    };

    let dir = utils::destination("clone", &home, args.dir.as_deref(), configured.as_deref())?;

    output::note(format!("cloning {} into {}", args.url, dir.display()));
    let cloned = git::clone(&dir, &args.url, lfs)?;

    let mut current = state::load()?;
    current.set_repo(dir.clone());
    state::save(&current)?;

    if cloned == git::Cloned::Empty {
        output::warn("the cloned repository is empty");
    }

    output::note(format!("cloned {}", dir.display()));
    utils::offer_definitions("clone", &dir);

    offer_bootstrap(&dir)
}

fn offer_bootstrap(repo: &Path) -> Result<()> {
    let path = lua::bootstrap_path("clone", repo)?;
    if !path.is_file() {
        return Ok(());
    }

    let question = format!("clone: found {}. Run it now?", path.display());
    if !output::offer("clone", &question)? {
        return Ok(());
    }

    let loaded = lua::load_config()?;
    utils::ask_missing("clone", utils::configured("clone", &loaded)?.classes())?;

    lua::run_bootstrap("clone", repo, &loaded)
}
