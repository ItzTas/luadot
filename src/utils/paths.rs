use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use etcetera::base_strategy::{BaseStrategy, Xdg};

use super::constants::{APP_DIR, HOME_PREFIX, ROOT_PREFIX};

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
    let normalized = normalize(outside);
    if let Ok(relative) = normalized.strip_prefix(home) {
        return Ok(Path::new(HOME_PREFIX).join(relative));
    }

    let Ok(relative) = normalized.strip_prefix("/") else {
        bail!("{} is not an absolute path", normalized.display());
    };
    Ok(Path::new(ROOT_PREFIX).join(relative))
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

    target(home, relative)
}

fn target(home: &Path, relative: &Path) -> Result<PathBuf> {
    if let Ok(rest) = relative.strip_prefix(HOME_PREFIX) {
        return Ok(home.join(rest));
    }
    if let Ok(rest) = relative.strip_prefix(ROOT_PREFIX) {
        return Ok(Path::new("/").join(rest));
    }

    bail!(
        "{} is outside {HOME_PREFIX}/ and {ROOT_PREFIX}/",
        relative.display()
    )
}

pub fn is_managed(relative: &Path) -> bool {
    relative.starts_with(HOME_PREFIX) || relative.starts_with(ROOT_PREFIX)
}

pub fn is_root(relative: &Path) -> bool {
    relative.starts_with(ROOT_PREFIX)
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
    fn expand_resolves_a_path_against_the_home_directory() {
        let home = Path::new("/home/u");

        assert_eq!(expand(home, Path::new("~")), PathBuf::from("/home/u"));
        assert_eq!(
            expand(home, Path::new("~/backups")),
            PathBuf::from("/home/u/backups")
        );
        assert_eq!(
            expand(home, Path::new("backups")),
            PathBuf::from("/home/u/backups")
        );
        assert_eq!(
            expand(home, Path::new("/data/backups")),
            PathBuf::from("/data/backups")
        );
    }

    #[test]
    fn repo_path_mirrors_home_under_the_home_prefix() {
        let dest = repo_path(
            Path::new("/home/u"),
            Path::new("/repo"),
            Path::new("/home/u/.config/nvim/init.lua"),
        )
        .unwrap();
        assert_eq!(dest, PathBuf::from("/repo/home/.config/nvim/init.lua"));
    }

    #[test]
    fn repo_path_mirrors_the_rest_of_the_system_under_the_root_prefix() {
        let dest = repo_path(
            Path::new("/home/u"),
            Path::new("/repo"),
            Path::new("/etc/pacman.conf"),
        )
        .unwrap();
        assert_eq!(dest, PathBuf::from("/repo/root/etc/pacman.conf"));
    }

    #[test]
    fn repo_path_resolves_dotdot_before_choosing_a_prefix() {
        let dest = repo_path(
            Path::new("/home/u"),
            Path::new("/repo"),
            Path::new("/home/u/../../etc/passwd"),
        )
        .unwrap();
        assert_eq!(dest, PathBuf::from("/repo/root/etc/passwd"));
    }

    #[test]
    fn repo_path_rejects_a_relative_path() {
        let err = repo_path(
            Path::new("/home/u"),
            Path::new("/repo"),
            Path::new("../outside"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("is not an absolute path"));
    }

    #[test]
    fn system_path_inverts_repo_path() {
        let home = Path::new("/home/u");
        let repo = Path::new("/repo");

        assert_eq!(
            system_path(home, repo, Path::new("/repo/home/.config/nvim/init.lua")).unwrap(),
            PathBuf::from("/home/u/.config/nvim/init.lua")
        );
        assert_eq!(
            system_path(home, repo, Path::new("/repo/root/etc/pacman.conf")).unwrap(),
            PathBuf::from("/etc/pacman.conf")
        );
    }

    #[test]
    fn system_path_rejects_a_file_outside_the_prefixes() {
        let err = system_path(
            Path::new("/home/u"),
            Path::new("/repo"),
            Path::new("/repo/README.md"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("is outside home/ and root/"));
    }

    #[test]
    fn managed_relative_names_the_prefixed_path() {
        let home = Path::new("/home/u");

        assert_eq!(
            managed_relative(home, Path::new("/home/u/.vimrc")).unwrap(),
            PathBuf::from("home/.vimrc")
        );
        assert_eq!(
            managed_relative(home, Path::new("/etc/hosts")).unwrap(),
            PathBuf::from("root/etc/hosts")
        );
    }

    #[test]
    fn the_prefixes_are_told_apart_by_component() {
        assert!(is_managed(Path::new("home/.vimrc")));
        assert!(is_managed(Path::new("root/etc/hosts")));
        assert!(!is_managed(Path::new("README.md")));
        assert!(!is_managed(Path::new("homework/.vimrc")));

        assert!(is_root(Path::new("root/etc/hosts")));
        assert!(!is_root(Path::new("home/.vimrc")));
        assert!(!is_root(Path::new("rooted/file")));
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
