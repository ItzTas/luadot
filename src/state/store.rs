use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::State;
use crate::utils;

pub fn load() -> Result<State> {
    load_from(&state_path()?)
}

pub fn save(state: &State) -> Result<()> {
    save_to(&state_path()?, state)
}

fn load_from(path: &Path) -> Result<State> {
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents)
            .with_context(|| format!("state: failed to parse {}", path.display())),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(State::default()),
        Err(err) => Err(err).with_context(|| format!("state: failed to read {}", path.display())),
    }
}

fn save_to(path: &Path, state: &State) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("state: failed to create {}", parent.display()))?;
    }
    let contents = serde_json::to_string_pretty(state).context("state: failed to serialize")?;
    std::fs::write(path, contents)
        .with_context(|| format!("state: failed to write {}", path.display()))
}

fn state_path() -> Result<PathBuf> {
    Ok(utils::data_dir()?.join("state.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_loads_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        assert!(load_from(&path).unwrap().repo.is_none());
    }

    #[test]
    fn save_then_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/state.json");

        let state = State {
            repo: Some(PathBuf::from("/x/y/repo")),
        };
        save_to(&path, &state).unwrap();

        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.repo, Some(PathBuf::from("/x/y/repo")));
    }

    #[test]
    fn invalid_json_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        std::fs::write(&path, "not json").unwrap();
        assert!(load_from(&path).is_err());
    }
}
