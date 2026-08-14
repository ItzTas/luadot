use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use etcetera::base_strategy::{BaseStrategy, Xdg};

use super::constants::APP_DIR;

pub fn data_dir() -> Result<PathBuf> {
    Ok(app_dir(&base()?.data_dir()))
}

pub fn config_dir() -> Result<PathBuf> {
    Ok(app_dir(&base()?.config_dir()))
}

pub fn home_dir() -> Result<PathBuf> {
    Ok(base()?.home_dir().to_path_buf())
}

fn base() -> Result<Xdg> {
    Xdg::new().context("failed to locate your home directory")
}

fn app_dir(root: &Path) -> PathBuf {
    root.join(APP_DIR)
}

pub fn repo_path(home: &Path, repo: &Path, outside: &Path) -> Result<PathBuf> {
    let normalized = normalize(outside);
    let Ok(relative) = normalized.strip_prefix(home) else {
        bail!(
            "{} is not inside your home directory {}",
            normalized.display(),
            home.display()
        );
    };

    Ok(repo.join(relative))
}

pub fn system_path(home: &Path, repo: &Path, inside: &Path) -> Result<PathBuf> {
    let Ok(relative) = inside.strip_prefix(repo) else {
        bail!(
            "{} is not inside the repository {}",
            inside.display(),
            repo.display()
        );
    };

    Ok(home.join(relative))
}

pub fn relative<'a>(repo: &Path, file: &'a Path) -> &'a Path {
    file.strip_prefix(repo).unwrap_or(file)
}

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::RootDir => out.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => ascend(&mut out),
            Component::Normal(segment) => out.push(segment),
        }
    }
    out
}

fn ascend(out: &mut PathBuf) {
    if matches!(out.components().next_back(), Some(Component::Normal(_))) {
        out.pop();
        return;
    }
    if out.has_root() {
        return;
    }
    out.push("..");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_dir_is_nested_under_the_given_root() {
        assert_eq!(
            app_dir(Path::new("/home/u/.local/share")),
            PathBuf::from("/home/u/.local/share/luadot")
        );
        assert_eq!(
            app_dir(Path::new("/home/u/.config")),
            PathBuf::from("/home/u/.config/luadot")
        );
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
    fn relative_strips_the_repository_prefix() {
        assert_eq!(
            relative(Path::new("/repo"), Path::new("/repo/.config/nvim/init.lua")),
            Path::new(".config/nvim/init.lua")
        );
    }

    #[test]
    fn relative_keeps_paths_outside_the_repository() {
        assert_eq!(
            relative(Path::new("/repo"), Path::new("/tmp/x")),
            Path::new("/tmp/x")
        );
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
