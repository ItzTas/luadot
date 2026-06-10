use std::env;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};

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

pub fn home_dir() -> Result<PathBuf> {
    resolve_home(env::var_os("HOME"))
}

fn resolve_home(home: Option<OsString>) -> Result<PathBuf> {
    match home {
        Some(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        _ => bail!("HOME is not set"),
    }
}

pub fn repo_path(home: &Path, repo: &Path, outside: &Path) -> Result<PathBuf> {
    let normalized = normalize(outside);
    match normalized.strip_prefix(home) {
        Ok(rel) => Ok(repo.join(rel)),
        Err(_) => bail!(
            "{} is not inside your home directory {}",
            normalized.display(),
            home.display()
        ),
    }
}

pub fn system_path(home: &Path, repo: &Path, inside: &Path) -> Result<PathBuf> {
    match inside.strip_prefix(repo) {
        Ok(rel) => Ok(home.join(rel)),
        Err(_) => bail!(
            "{} is not inside the repository {}",
            inside.display(),
            repo.display()
        ),
    }
}

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::RootDir => out.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(out.components().next_back(), Some(Component::Normal(_))) {
                    out.pop();
                } else if !out.has_root() {
                    out.push("..");
                }
            }
            Component::Normal(segment) => out.push(segment),
        }
    }
    out
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

    #[test]
    fn resolve_home_uses_home_when_set() {
        let home = resolve_home(Some("/home/u".into())).unwrap();
        assert_eq!(home, PathBuf::from("/home/u"));
    }

    #[test]
    fn resolve_home_errors_when_unset_or_empty() {
        assert!(resolve_home(None).is_err());
        assert!(resolve_home(Some(OsString::new())).is_err());
    }

    #[test]
    fn normalize_resolves_dot_and_dotdot() {
        assert_eq!(normalize(Path::new("/a/b/../c")), PathBuf::from("/a/c"));
        assert_eq!(normalize(Path::new("/a/./b")), PathBuf::from("/a/b"));
        assert_eq!(normalize(Path::new("/a/b/..")), PathBuf::from("/a"));
        assert_eq!(normalize(Path::new("/..")), PathBuf::from("/"));
        assert_eq!(normalize(Path::new("a/../b")), PathBuf::from("b"));
        assert_eq!(normalize(Path::new("../a")), PathBuf::from("../a"));
    }

    #[test]
    fn repo_path_mirrors_home_layout() {
        let dest = repo_path(
            Path::new("/home/u"),
            Path::new("/repo"),
            Path::new("/home/u/.config/nvim/init.lua"),
        )
        .unwrap();
        assert_eq!(dest, PathBuf::from("/repo/.config/nvim/init.lua"));
    }

    #[test]
    fn repo_path_rejects_paths_outside_home() {
        assert!(
            repo_path(
                Path::new("/home/u"),
                Path::new("/repo"),
                Path::new("/etc/passwd")
            )
            .is_err()
        );
    }

    #[test]
    fn repo_path_rejects_dotdot_escape() {
        let err = repo_path(
            Path::new("/home/u"),
            Path::new("/repo"),
            Path::new("/home/u/../etc/passwd"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("not inside your home directory"));
    }

    #[test]
    fn system_path_inverts_repo_path() {
        let dest = system_path(
            Path::new("/home/u"),
            Path::new("/repo"),
            Path::new("/repo/.config/nvim/init.lua"),
        )
        .unwrap();
        assert_eq!(dest, PathBuf::from("/home/u/.config/nvim/init.lua"));
    }

    #[test]
    fn system_path_rejects_paths_outside_repo() {
        assert!(
            system_path(
                Path::new("/home/u"),
                Path::new("/repo"),
                Path::new("/tmp/x")
            )
            .is_err()
        );
    }
}
