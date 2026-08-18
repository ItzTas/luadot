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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_directory_is_empty() {
        let dir = tempfile::tempdir().unwrap();

        require_empty("clone", &dir.path().join("repo")).unwrap();
    }

    #[test]
    fn an_existing_empty_directory_passes() {
        let dir = tempfile::tempdir().unwrap();

        require_empty("init", dir.path()).unwrap();
    }

    #[test]
    fn a_directory_holding_something_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("existing.txt"), "data").unwrap();

        let err = require_empty("clone", dir.path()).unwrap_err().to_string();

        assert!(err.contains("clone: "));
        assert!(err.contains("is not empty"));
    }
}
