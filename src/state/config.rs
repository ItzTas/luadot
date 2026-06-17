use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    config: StateConfig,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct StateConfig {
    repo: Option<PathBuf>,
}

impl State {
    pub fn repo(&self) -> Option<&Path> {
        self.config.repo()
    }

    pub fn set_repo(&mut self, repo: PathBuf) {
        self.config.set_repo(repo);
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        self.config.get(key)
    }
}

impl StateConfig {
    fn repo(&self) -> Option<&Path> {
        self.repo.as_deref()
    }

    fn set_repo(&mut self, repo: PathBuf) {
        self.repo = Some(repo);
    }

    #[allow(dead_code)]
    fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let value = serde_json::to_value(self).context("state: failed to serialize")?;
        Ok(value.get(key).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_repo() {
        assert!(State::default().repo().is_none());
    }

    #[test]
    fn serde_roundtrip_with_repo() {
        let mut state = State::default();
        state.set_repo(PathBuf::from("/a/b"));

        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, r#"{"config":{"repo":"/a/b"}}"#);

        let back: State = serde_json::from_str(&json).unwrap();
        assert_eq!(back.repo(), state.repo());
    }
}
