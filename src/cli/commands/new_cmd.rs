use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;

use super::super::constants::TEMPLATE_SKELETON;
use crate::files;
use crate::lua::TEMPLATE_FILE;
use crate::output;
use crate::utils::{self, Workspace};

#[derive(Debug, Args)]
pub struct NewArgs {
    #[arg(value_name = "PATH")]
    pub path: String,
    #[arg(
        short,
        long,
        help = "Create a standalone template file instead of a template directory"
    )]
    pub file: bool,
}

pub fn new_cmd(args: NewArgs) -> Result<()> {
    let Workspace { home, repo, .. } = utils::workspace("new")?;

    let template = destination(&home, &repo, &args.path)?;

    create(&template, args.file)?;
    output::note(format!("created {}", template.display()));

    Ok(())
}

fn destination(home: &Path, repo: &Path, arg: &str) -> Result<PathBuf> {
    let target = absolute(home, arg)?;
    let managed = utils::repo_path(home, repo, &target)?;

    if files::template_target(&managed).is_some() {
        return Ok(managed);
    }

    files::template_dir(&managed).with_context(|| format!("new: {arg} does not name a template"))
}

fn absolute(home: &Path, arg: &str) -> Result<PathBuf> {
    let path = Path::new(arg);
    if path.starts_with("~") {
        return Ok(utils::expand(home, path));
    }

    std::path::absolute(path).with_context(|| format!("new: invalid path {arg}"))
}

fn create(template: &Path, file: bool) -> Result<()> {
    if std::fs::symlink_metadata(template).is_ok() {
        bail!("new: {} already exists", template.display());
    }
    if let Some(target) = files::template_target(template)
        && std::fs::symlink_metadata(&target).is_ok()
    {
        bail!(
            "new: {} is already in the repository; run `luadot rm` on it first",
            target.display()
        );
    }

    match file {
        true => create_file(template),
        false => create_dir(template),
    }
}

fn create_file(template: &Path) -> Result<()> {
    create_parent(template)?;

    std::fs::write(template, "")
        .with_context(|| format!("new: failed to create {}", template.display()))
}

fn create_dir(template: &Path) -> Result<()> {
    std::fs::create_dir_all(template)
        .with_context(|| format!("new: failed to create {}", template.display()))?;

    let script = template.join(TEMPLATE_FILE);
    std::fs::write(&script, TEMPLATE_SKELETON)
        .with_context(|| format!("new: failed to create {}", script.display()))
}

fn create_parent(template: &Path) -> Result<()> {
    let Some(parent) = template.parent() else {
        return Ok(());
    };

    std::fs::create_dir_all(parent)
        .with_context(|| format!("new: failed to create {}", parent.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home_and_repo() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repo = root.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&repo).unwrap();

        (root, home, repo)
    }

    fn arg(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn a_name_without_the_suffix_gets_it() {
        let (_root, home, repo) = home_and_repo();

        assert_eq!(
            destination(&home, &repo, &arg(&home.join(".zshrc"))).unwrap(),
            repo.join("home/.zshrc.luadot")
        );
    }

    #[test]
    fn a_name_already_carrying_the_suffix_keeps_it() {
        let (_root, home, repo) = home_and_repo();

        assert_eq!(
            destination(&home, &repo, &arg(&home.join(".zshrc.luadot"))).unwrap(),
            repo.join("home/.zshrc.luadot")
        );
    }

    #[test]
    fn a_nested_path_mirrors_the_home_layout() {
        let (_root, home, repo) = home_and_repo();

        assert_eq!(
            destination(&home, &repo, &arg(&home.join(".config/nvim/init.lua"))).unwrap(),
            repo.join("home/.config/nvim/init.lua.luadot")
        );
    }

    #[test]
    fn a_tilde_stands_for_the_home_directory() {
        let (_root, home, repo) = home_and_repo();

        assert_eq!(
            destination(&home, &repo, "~/.zshrc").unwrap(),
            repo.join("home/.zshrc.luadot")
        );
    }

    #[test]
    fn a_system_path_lands_under_the_root_prefix() {
        let (_root, home, repo) = home_and_repo();

        assert_eq!(
            destination(&home, &repo, "/etc/zsh/zshrc").unwrap(),
            repo.join("root/etc/zsh/zshrc.luadot")
        );
    }

    #[test]
    fn a_directory_template_holds_the_script_it_needs() {
        let (_root, _home, repo) = home_and_repo();
        let template = repo.join(".zshrc.luadot");

        create(&template, false).unwrap();

        assert!(template.is_dir());
        assert_eq!(
            std::fs::read_to_string(template.join(TEMPLATE_FILE)).unwrap(),
            TEMPLATE_SKELETON
        );
    }

    #[test]
    fn a_standalone_template_is_an_empty_file() {
        let (_root, _home, repo) = home_and_repo();
        let template = repo.join(".zprofile.luadot");

        create(&template, true).unwrap();

        assert!(template.is_file());
        assert_eq!(std::fs::read_to_string(&template).unwrap(), "");
    }

    #[test]
    fn a_standalone_template_gets_the_directories_leading_to_it() {
        let (_root, _home, repo) = home_and_repo();
        let template = repo.join(".config/nvim/init.lua.luadot");

        create(&template, true).unwrap();

        assert!(template.is_file());
    }

    #[test]
    fn an_existing_template_is_left_alone() {
        let (_root, _home, repo) = home_and_repo();
        let template = repo.join(".zshrc.luadot");
        std::fs::create_dir_all(&template).unwrap();
        std::fs::write(template.join(TEMPLATE_FILE), "kept").unwrap();

        let err = create(&template, false).unwrap_err().to_string();

        assert!(err.contains("new: "));
        assert!(err.contains("already exists"));
        assert_eq!(
            std::fs::read_to_string(template.join(TEMPLATE_FILE)).unwrap(),
            "kept"
        );
    }

    #[test]
    fn a_file_the_repository_already_manages_is_reported() {
        let (_root, _home, repo) = home_and_repo();
        std::fs::write(repo.join(".zshrc"), "managed").unwrap();

        let err = create(&repo.join(".zshrc.luadot"), false)
            .unwrap_err()
            .to_string();

        assert!(err.contains("new: "));
        assert!(err.contains("already in the repository"));
    }
}
