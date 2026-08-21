use std::path::{Path, PathBuf};

use crate::lua::from_template;

pub fn template(root: &Path) -> PathBuf {
    let dir = root.join(".zshrc.luadot");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn error(dir: &Path, source: &str) -> String {
    format!("{:#}", from_template(dir, source).unwrap_err())
}
