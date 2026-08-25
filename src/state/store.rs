use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::debug;

use super::State;
use crate::files;
use crate::utils;

pub fn load() -> Result<State> {
    load_from(&state_path()?)
}

#[allow(dead_code)]
pub fn lcget(key: &str) -> Result<Option<serde_json::Value>> {
    load()?.get(key)
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
    let contents = serde_json::to_string_pretty(state).context("state: failed to serialize")?;
    files::replace_contents("state", path, contents.as_bytes())?;
    debug!(path = %path.display(), "saved the state");
    Ok(())
}

fn state_path() -> Result<PathBuf> {
    Ok(utils::data_dir()?.join("state.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_then_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/state.json");

        let mut state = State::default();
        state.set_repo(PathBuf::from("/x/y/repo"));
        save_to(&path, &state).unwrap();

        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.repo(), Some(Path::new("/x/y/repo")));
    }
}
