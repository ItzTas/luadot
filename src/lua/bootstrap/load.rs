use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::constants::BOOTSTRAP_FILE;
use crate::lua::ld::{Paths, Surface};
use crate::lua::script::run_script;
use crate::state::{self, Classes};
use crate::utils;

pub fn bootstrap_path(command: &str, repo: &Path) -> Result<PathBuf> {
    resolve(&utils::home_dir()?, &utils::config_dir()?, repo)
        .with_context(|| format!("{command}: failed to locate the bootstrap file"))
}

pub fn run_bootstrap(command: &str, repo: &Path) -> Result<()> {
    let home = utils::home_dir()?;
    let config = utils::config_dir()?;
    let path = resolve(&home, &config, repo)
        .with_context(|| format!("{command}: failed to locate the bootstrap file"))?;
    let classes = state::load()?.classes().clone();

    run_file(command, &path, &home, &config, repo, &classes)
}

fn resolve(home: &Path, config: &Path, repo: &Path) -> Result<PathBuf> {
    utils::repo_path(home, repo, &config.join(BOOTSTRAP_FILE))
}

fn run_file(
    command: &str,
    path: &Path,
    home: &Path,
    config: &Path,
    repo: &Path,
    classes: &Classes,
) -> Result<()> {
    let modules = path
        .parent()
        .with_context(|| format!("{command}: {} has no parent directory", path.display()))?;
    let paths = Paths::new(home, config).with_repo(Some(repo));

    run_script(command, Surface::Bootstrap, path, modules, &paths, classes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::constants::MODULES_DIR;

    fn write_bootstrap(home: &Path, repo: &Path, source: &str) -> PathBuf {
        let config = home.join(".config/luadot");
        let path = resolve(home, &config, repo).unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, source).unwrap();
        path
    }

    #[test]
    fn resolve_mirrors_the_config_location_into_the_repo() {
        let path = resolve(
            Path::new("/home/u"),
            Path::new("/home/u/.config/luadot"),
            Path::new("/data/repo"),
        )
        .unwrap();

        assert_eq!(
            path,
            PathBuf::from("/data/repo/home/.config/luadot/bootstrap.lua")
        );
    }

    #[test]
    fn resolve_maps_a_config_outside_home_under_root() {
        let path = resolve(
            Path::new("/home/u"),
            Path::new("/etc/luadot"),
            Path::new("/data/repo"),
        )
        .unwrap();

        assert_eq!(
            path,
            PathBuf::from("/data/repo/root/etc/luadot/bootstrap.lua")
        );
    }

    #[test]
    fn runs_the_script_with_the_bootstrap_api_and_modules() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let config = home.join(".config/luadot");
        let path = write_bootstrap(
            &home,
            &repo,
            r#"
            local greeting = require("greeting")
            local out = assert(io.open(ld.path.repo .. "/out.txt", "w"))
            out:write(greeting)
            out:close()
            "#,
        );
        let modules = path.parent().unwrap().join(MODULES_DIR);
        std::fs::create_dir_all(&modules).unwrap();
        std::fs::write(modules.join("greeting.lua"), r#"return "hello""#).unwrap();

        run_file(
            "bootstrap",
            &path,
            &home,
            &config,
            &repo,
            &Classes::default(),
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(repo.join("out.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn the_script_reads_the_classes_of_the_machine() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let config = home.join(".config/luadot");
        let path = write_bootstrap(
            &home,
            &repo,
            r#"
            local out = assert(io.open(ld.path.repo .. "/class.txt", "w"))
            out:write(ld.class.get("form-factor"))
            out:close()
            "#,
        );
        let mut classes = Classes::default();
        classes.set("form-factor", "laptop");

        run_file("bootstrap", &path, &home, &config, &repo, &classes).unwrap();

        assert_eq!(
            std::fs::read_to_string(repo.join("class.txt")).unwrap(),
            "laptop"
        );
    }

    #[test]
    fn a_missing_file_reports_the_command() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(BOOTSTRAP_FILE);

        let err = format!(
            "{:#}",
            run_file(
                "clone",
                &path,
                Path::new("/home/u"),
                Path::new("/home/u/.config/luadot"),
                dir.path(),
                &Classes::default(),
            )
            .unwrap_err()
        );

        assert!(err.contains("clone: failed to read"));
    }

    #[test]
    fn a_broken_script_reports_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let config = home.join(".config/luadot");
        let path = write_bootstrap(&home, &repo, "ld.cmd(");

        let err = format!(
            "{:#}",
            run_file(
                "bootstrap",
                &path,
                &home,
                &config,
                &repo,
                &Classes::default()
            )
            .unwrap_err()
        );

        assert!(err.contains("bootstrap: failed to run"));
    }
}
