use anyhow::{Result, bail};

use crate::{lua, utils};

pub fn bootstrap_cmd() -> Result<()> {
    let loaded = lua::load_config()?;
    let config = utils::configured("bootstrap", &loaded)?;
    let repo = utils::require_repo("bootstrap", config.repo_dir())?;
    let path = lua::bootstrap_path("bootstrap", &repo)?;

    if !path.is_file() {
        bail!("bootstrap: no bootstrap file at {}", path.display());
    }

    utils::ask_missing("bootstrap", config.classes())?;

    drop(config);

    lua::run_bootstrap("bootstrap", &repo, &loaded)
}
