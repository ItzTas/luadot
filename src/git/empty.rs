use std::path::Path;

use anyhow::{Context, Result, bail};

pub(super) fn require_empty(command: &str, dir: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    let mut entries = std::fs::read_dir(dir)
        .with_context(|| format!("{command}: failed to read {}", dir.display()))?;
    if entries.next().is_some() {
        bail!("{command}: destination {} is not empty", dir.display());
    }

    Ok(())
}
