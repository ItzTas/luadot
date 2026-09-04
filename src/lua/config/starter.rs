use std::path::{Path, PathBuf};

use anyhow::Result;

use super::constants::STARTER;
use crate::files;

pub fn place(command: &str, path: &Path) -> Result<Option<PathBuf>> {
    if path.exists() {
        return Ok(None);
    }

    files::replace_contents(command, path, STARTER.as_bytes())?;

    Ok(Some(path.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::{Config, from_source};

    #[test]
    fn an_empty_directory_gets_the_starter() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("luadot").join("config.lua");

        let placed = place("init", &path).unwrap();

        assert_eq!(placed, Some(path.clone()));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), STARTER);
    }

    #[test]
    fn an_existing_configuration_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.lua");
        std::fs::write(&path, "ld.opt.link(\"copy\")\n").unwrap();

        let placed = place("config", &path).unwrap();

        assert_eq!(placed, None);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "ld.opt.link(\"copy\")\n"
        );
    }

    #[test]
    fn the_starter_runs_silently() {
        let config = from_source(STARTER).unwrap();

        assert_eq!(config.link(), Config::default().link());
        assert!(config.rules().is_empty());
    }
}
