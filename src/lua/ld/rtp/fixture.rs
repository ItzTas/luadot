use std::path::{Path, PathBuf};

use crate::lua::constants::MODULES_DIR;

pub fn plugin(root: &Path, name: &str, source: &str) -> PathBuf {
    let dir = root.join(name);
    let modules = dir.join(MODULES_DIR);
    std::fs::create_dir_all(&modules).unwrap();
    std::fs::write(modules.join(format!("{name}.lua")), source).unwrap();

    dir
}
