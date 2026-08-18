use std::path::Path;

use anyhow::{Context, Result};
use tracing::debug;

use super::empty::require_empty;

pub fn init(dir: &Path) -> Result<()> {
    debug!(dir = %dir.display(), "initializing");
    require_empty("init", dir)?;

    std::fs::create_dir_all(dir)
        .with_context(|| format!("init: failed to create {}", dir.display()))?;

    gix::init(dir)
        .with_context(|| format!("init: failed to create a repository in {}", dir.display()))?;

    debug!(dir = %dir.display(), "initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_repository_in_a_missing_directory() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("nested/repo");

        init(&target).unwrap();

        assert!(target.join(".git").is_dir());
    }
}
