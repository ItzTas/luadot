use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Classes(BTreeMap<String, String>);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    repo: Option<PathBuf>,
    #[serde(default)]
    classes: Classes,
}

impl Classes {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.get(name).map(String::as_str)
    }

    pub fn set(&mut self, name: &str, value: &str) {
        self.0.insert(name.to_string(), value.to_string());
    }

    pub fn unset(&mut self, name: &str) -> bool {
        self.0.remove(name).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl State {
    pub fn repo(&self) -> Option<&Path> {
        self.repo.as_deref()
    }

    pub fn set_repo(&mut self, repo: PathBuf) {
        self.repo = Some(repo);
    }

    pub fn classes(&self) -> &Classes {
        &self.classes
    }

    pub fn class(&self, name: &str) -> Option<&str> {
        self.classes.get(name)
    }

    pub fn set_class(&mut self, name: &str, value: &str) {
        self.classes.set(name, value);
    }

    pub fn unset_class(&mut self, name: &str) -> bool {
        self.classes.unset(name)
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
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
    fn default_has_no_class() {
        let state = State::default();

        assert!(state.classes().is_empty());
        assert_eq!(state.class("form-factor"), None);
    }

    #[test]
    fn serde_roundtrip_with_repo() {
        let mut state = State::default();
        state.set_repo(PathBuf::from("/a/b"));

        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, r#"{"repo":"/a/b","classes":{}}"#);

        let back: State = serde_json::from_str(&json).unwrap();
        assert_eq!(back.repo(), state.repo());
    }

    #[test]
    fn serde_roundtrip_with_a_class() {
        let mut state = State::default();
        state.set_class("form-factor", "laptop");

        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, r#"{"repo":null,"classes":{"form-factor":"laptop"}}"#);

        let back: State = serde_json::from_str(&json).unwrap();
        assert_eq!(back.class("form-factor"), Some("laptop"));
    }

    #[test]
    fn a_state_written_before_classes_still_loads() {
        let state: State = serde_json::from_str(r#"{"repo":"/a/b"}"#).unwrap();

        assert_eq!(state.repo(), Some(Path::new("/a/b")));
        assert!(state.classes().is_empty());
    }

    #[test]
    fn setting_a_class_twice_keeps_the_last_answer() {
        let mut state = State::default();
        state.set_class("form-factor", "desktop");
        state.set_class("form-factor", "laptop");

        assert_eq!(state.class("form-factor"), Some("laptop"));
    }

    #[test]
    fn unsetting_reports_whether_it_was_set() {
        let mut state = State::default();
        state.set_class("form-factor", "laptop");

        assert!(state.unset_class("form-factor"));
        assert!(!state.unset_class("form-factor"));
        assert_eq!(state.class("form-factor"), None);
    }

    #[test]
    fn classes_are_iterated_by_name() {
        let mut state = State::default();
        state.set_class("form-factor", "laptop");
        state.set_class("email", "me@example.com");

        assert_eq!(
            state.classes().iter().collect::<Vec<_>>(),
            [("email", "me@example.com"), ("form-factor", "laptop")]
        );
    }

    #[test]
    fn get_reads_a_top_level_key() {
        let mut state = State::default();
        state.set_repo(PathBuf::from("/a/b"));

        assert_eq!(state.get("repo").unwrap().unwrap(), "/a/b");
        assert!(state.get("missing").unwrap().is_none());
    }
}
