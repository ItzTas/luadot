use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    pub repo: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_repo() {
        assert!(State::default().repo.is_none());
    }

    #[test]
    fn serde_roundtrip_with_repo() {
        let state = State {
            repo: Some(PathBuf::from("/a/b")),
        };

        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, r#"{"repo":"/a/b"}"#);

        let back: State = serde_json::from_str(&json).unwrap();
        assert_eq!(back.repo, state.repo);
    }
}
