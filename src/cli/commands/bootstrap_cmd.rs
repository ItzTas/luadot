use anyhow::{Result, bail};

use crate::{lua, utils};

pub fn bootstrap_cmd() -> Result<()> {
    let repo = utils::require_repo("bootstrap")?;
    let path = lua::bootstrap_path("bootstrap", &repo)?;

    if !path.is_file() {
        bail!("bootstrap: no bootstrap file at {}", path.display());
    }

    utils::ask_missing("bootstrap", lua::load_config()?.classes())?;

    lua::run_bootstrap("bootstrap", &repo)
}
