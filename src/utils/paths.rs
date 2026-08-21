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

pub fn expand(home: &Path, path: &Path) -> PathBuf {
    if let Ok(rest) = path.strip_prefix("~") {
        return home.join(rest);
    }
    if path.is_absolute() {
        return path.to_path_buf();
    }

    home.join(path)
}

pub fn managed_relative(home: &Path, outside: &Path) -> Result<PathBuf> {
    let Ok(relative) = normalize(outside).strip_prefix(home).map(Path::to_path_buf) else {
        bail!("outside your home directory {}", home.display());
    };

    Ok(relative)
}

pub fn repo_path(home: &Path, repo: &Path, outside: &Path) -> Result<PathBuf> {
    Ok(repo.join(managed_relative(home, outside)?))
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
    fn normalize_resolves_dot_and_dotdot() {
        assert_eq!(normalize(Path::new("/a/b/../c")), PathBuf::from("/a/c"));
        assert_eq!(normalize(Path::new("/a/./b")), PathBuf::from("/a/b"));
        assert_eq!(normalize(Path::new("/a/b/..")), PathBuf::from("/a"));
        assert_eq!(normalize(Path::new("/..")), PathBuf::from("/"));
        assert_eq!(normalize(Path::new("a/../b")), PathBuf::from("b"));
        assert_eq!(normalize(Path::new("../a")), PathBuf::from("../a"));
    }

    #[test]
    fn repo_path_resolves_dotdot_before_checking_the_home_directory() {
        let dest = repo_path(
            Path::new("/home/u"),
            Path::new("/repo"),
            Path::new("/home/u/.config/../.vimrc"),
        )
        .unwrap();
        assert_eq!(dest, PathBuf::from("/repo/.vimrc"));

        let err = repo_path(
            Path::new("/home/u"),
            Path::new("/repo"),
            Path::new("/home/u/../../etc/passwd"),
        )
        .unwrap_err();
        assert_eq!(err.to_string(), "outside your home directory /home/u");
    }

    #[test]
    fn system_path_inverts_repo_path() {
        let home = Path::new("/home/u");
        let repo = Path::new("/repo");

        assert_eq!(
            system_path(home, repo, Path::new("/repo/.config/nvim/init.lua")).unwrap(),
            PathBuf::from("/home/u/.config/nvim/init.lua")
        );
    }
}
