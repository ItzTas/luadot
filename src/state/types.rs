use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    pub repo: Option<PathBuf>,
}
