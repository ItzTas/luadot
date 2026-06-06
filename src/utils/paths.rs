use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{Context, Result};

pub fn data_dir() -> Result<PathBuf> {
    resolve_data_dir(env::var_os("XDG_DATA_HOME"), env::var_os("HOME"))
}

fn resolve_data_dir(xdg_data_home: Option<OsString>, home: Option<OsString>) -> Result<PathBuf> {
    const APP_DIR: &str = "luadot";
    const DEFAULT_DATA_DIR: &str = ".local/share";

    let base = match xdg_data_home {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => {
            let home = home.context("HOME is not set")?;
            PathBuf::from(home).join(DEFAULT_DATA_DIR)
        }
    };

    Ok(base.join(APP_DIR))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_xdg_data_home_when_set() {
        let dir = resolve_data_dir(Some("/data".into()), Some("/home/u".into())).unwrap();
        assert_eq!(dir, PathBuf::from("/data/luadot"));
    }

    #[test]
    fn falls_back_to_home_when_xdg_unset() {
        let dir = resolve_data_dir(None, Some("/home/u".into())).unwrap();
        assert_eq!(dir, PathBuf::from("/home/u/.local/share/luadot"));
    }

    #[test]
    fn empty_xdg_falls_back_to_home() {
        let dir = resolve_data_dir(Some(OsString::new()), Some("/home/u".into())).unwrap();
        assert_eq!(dir, PathBuf::from("/home/u/.local/share/luadot"));
    }

    #[test]
    fn errors_without_xdg_and_home() {
        assert!(resolve_data_dir(None, None).is_err());
    }
}
