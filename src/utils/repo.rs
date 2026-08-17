use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::paths::{expand, home_dir, repo_path};
use crate::state;

pub fn require_repo(command: &str, configured: Option<&Path>) -> Result<PathBuf> {
    let configured = match configured {
        Some(dir) => Some(expand(&home_dir()?, dir)),
        None => None,
    };

    resolve(command, configured, state::load()?.repo())
}

pub fn managed_path(command: &str, home: &Path, repo: &Path, arg: &str) -> Result<PathBuf> {
    let target =
        std::path::absolute(arg).with_context(|| format!("{command}: invalid path {arg}"))?;
    let managed = repo_path(home, repo, &target)?;

    if std::fs::symlink_metadata(&managed).is_err() {
        bail!(
            "{command}: {} is not managed by the repository",
            target.display()
        );
    }

    Ok(managed)
}

fn resolve(
    command: &str,
    configured: Option<PathBuf>,
    remembered: Option<&Path>,
) -> Result<PathBuf> {
    let Some(repo) = configured
        .clone()
        .or_else(|| remembered.map(Path::to_path_buf))
    else {
        bail!("{command}: no repository set; run `luadot clone <url>` first");
    };

    if repo.is_dir() {
        return Ok(repo);
    }

    if configured.is_some() {
        bail!(
            "{command}: repository {} does not exist; clone it there or fix `ld.opt.repo_dir`",
            repo.display()
        );
    }

    bail!(
        "{command}: repository {} does not exist; run `luadot clone <url>` first",
        repo.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_when_no_repository_is_set() {
        let err = resolve("add", None, None).unwrap_err().to_string();
        assert_eq!(
            err,
            "add: no repository set; run `luadot clone <url>` first"
        );
    }

    #[test]
    fn errors_when_the_repository_does_not_exist() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("repo");

        let err = resolve("git", None, Some(&missing))
            .unwrap_err()
            .to_string();
        assert_eq!(
            err,
            format!(
                "git: repository {} does not exist; run `luadot clone <url>` first",
                missing.display()
            )
        );
    }

    #[test]
    fn errors_when_the_configured_repository_does_not_exist() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("dotfiles");

        let err = resolve("status", Some(missing.clone()), Some(dir.path()))
            .unwrap_err()
            .to_string();

        assert_eq!(
            err,
            format!(
                "status: repository {} does not exist; clone it there or fix `ld.opt.repo_dir`",
                missing.display()
            )
        );
    }

    #[test]
    fn errors_when_the_repository_is_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("repo");
        std::fs::write(&file, "").unwrap();

        assert!(resolve("cd", None, Some(&file)).is_err());
    }

    #[test]
    fn returns_an_existing_repository() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(
            resolve("apply", None, Some(dir.path())).unwrap(),
            dir.path()
        );
    }

    #[test]
    fn the_configured_repository_wins_over_the_remembered_one() {
        let dir = tempfile::tempdir().unwrap();
        let configured = dir.path().join("dotfiles");
        let remembered = dir.path().join("repo");
        std::fs::create_dir_all(&configured).unwrap();
        std::fs::create_dir_all(&remembered).unwrap();

        assert_eq!(
            resolve("apply", Some(configured.clone()), Some(&remembered)).unwrap(),
            configured
        );
    }

    #[test]
    fn managed_path_maps_a_tracked_file_into_the_repository() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(repo.join("home")).unwrap();
        let tracked = repo.join("home/.bashrc");
        std::fs::write(&tracked, "data").unwrap();

        let arg = home.join(".bashrc").to_string_lossy().into_owned();

        assert_eq!(managed_path("rm", &home, &repo, &arg).unwrap(), tracked);
    }

    #[test]
    fn managed_path_errors_when_the_file_is_not_tracked() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();

        let arg = home.join(".bashrc").to_string_lossy().into_owned();
        let err = managed_path("rm", &home, &repo, &arg)
            .unwrap_err()
            .to_string();

        assert!(err.contains("rm: "));
        assert!(err.contains("is not managed by the repository"));
    }

    #[test]
    fn managed_path_accepts_a_dangling_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(repo.join("home")).unwrap();
        let tracked = repo.join("home/.bashrc");
        std::os::unix::fs::symlink(home.join(".bashrc"), &tracked).unwrap();

        let arg = home.join(".bashrc").to_string_lossy().into_owned();

        assert_eq!(managed_path("rm", &home, &repo, &arg).unwrap(), tracked);
    }
}
