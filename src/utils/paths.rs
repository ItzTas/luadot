use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};

pub fn data_dir() -> Result<PathBuf> {
    const APP_DIR: &str = "luadot";
    const DEFAULT_DATA_DIR: &str = ".local/share";

    let base = match env::var_os("XDG_DATA_HOME") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => {
            let home = env::var_os("HOME").context("HOME is not set")?;
            PathBuf::from(home).join(DEFAULT_DATA_DIR)
        }
    };

    Ok(base.join(APP_DIR))
}
