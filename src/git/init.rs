use std::path::Path;

use anyhow::{Context, Result};
use tracing::debug;

use super::empty::require_empty;
use super::lfs;

pub fn init(dir: &Path, lfs: bool) -> Result<()> {
    debug!(dir = %dir.display(), lfs, "initializing");
    require_empty("init", dir)?;

    std::fs::create_dir_all(dir)
        .with_context(|| format!("init: failed to create {}", dir.display()))?;

    gix::init(dir)
        .with_context(|| format!("init: failed to create a repository in {}", dir.display()))?;

    lfs::install("init", dir, lfs)?;

    debug!(dir = %dir.display(), "initialized");
    Ok(())
}
