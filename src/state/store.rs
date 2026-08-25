use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::debug;

use super::State;
use super::constants::{DATA_MODE, STATE_FILE, STATE_MODE};
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
    if let Some(dir) = path.parent() {
        files::private_dir("state", dir, DATA_MODE)?;
    }
    files::replace_file("state", path, |staged| {
        files::write_mode("state", staged, contents.as_bytes(), STATE_MODE)
    })?;
    debug!(path = %path.display(), "saved the state");
    Ok(())
}

fn state_path() -> Result<PathBuf> {
    Ok(utils::data_dir()?.join(STATE_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_state_and_its_directory_stay_private() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("share/luadot/state.json");

        save_to(&path, &State::default()).unwrap();

        let bits = |path: &Path| {
            std::fs::symlink_metadata(path)
                .unwrap()
                .permissions()
                .mode()
                & 0o7777
        };
        assert_eq!(bits(&path), STATE_MODE);
        assert_eq!(bits(path.parent().unwrap()), DATA_MODE);
    }

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
