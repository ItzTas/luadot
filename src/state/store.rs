use std::path::PathBuf;

use anyhow::{Context, Result};

use super::State;
use crate::utils;

pub fn load() -> Result<State> {
    let path = state_path()?;
    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents)
            .with_context(|| format!("state: failed to parse {}", path.display())),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(State::default()),
        Err(err) => Err(err).with_context(|| format!("state: failed to read {}", path.display())),
    }
}

pub fn save(state: &State) -> Result<()> {
    let path = state_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("state: failed to create {}", parent.display()))?;
    }
    let contents = serde_json::to_string_pretty(state).context("state: failed to serialize")?;
    std::fs::write(&path, contents)
        .with_context(|| format!("state: failed to write {}", path.display()))
}

fn state_path() -> Result<PathBuf> {
    Ok(utils::data_dir()?.join("state.json"))
}
