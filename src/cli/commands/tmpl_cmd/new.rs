use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;

use super::super::super::constants::TEMPLATE_SKELETON;
use super::super::{AddArgs, add_cmd};
use crate::files;
use crate::lua::{self, Placed, TEMPLATE_FILE};
use crate::output;
use crate::utils::{self, Workspace};

#[derive(Debug, Args)]
pub struct NewArgs {
    #[arg(value_name = "PATH", help = "The file the template produces")]
    pub path: String,
    #[arg(
        short,
        long,
        help = "Create a standalone template file instead of a template directory"
    )]
    pub file: bool,
}

pub fn new(args: NewArgs) -> Result<()> {
    let Workspace { home, repo, .. } = utils::workspace("tmpl new")?;

    let template = destination(&home, &args.path)?;
    let managed = utils::repo_path(&home, &repo, &template)?;

    create(&template, &managed, args.file)?;
    output::note(format!("created {}", template.display()));
    if !args.file {
        settings(&home, &template)?;
    }

    add_cmd(AddArgs {
        paths: vec![argument(&template)?],
    })?;
    report(&repo, &managed)
}

fn settings(home: &Path, template: &Path) -> Result<()> {
    match lua::point_at_definitions("tmpl new", template, home, &utils::config_dir()?)? {
        Placed::Written(path) => output::note(format!("created {}", path.display())),
        Placed::Merged(path) => output::note(format!("updated {}", path.display())),
        Placed::Kept(path, _) => output::warn(format!(
            "tmpl new: {} could not be parsed and was left alone",
            path.display()
        )),
    }

    Ok(())
}

fn destination(home: &Path, arg: &str) -> Result<PathBuf> {
    let target = absolute(home, arg)?;

    if files::is_template(&target) {
        return Ok(target);
    }

    files::template_dir(&target)
        .with_context(|| format!("tmpl new: {arg} does not name a template"))
}

fn argument(template: &Path) -> Result<String> {
    template
        .to_str()
        .map(str::to_owned)
        .with_context(|| format!("tmpl new: {} is not valid UTF-8", template.display()))
}

fn report(repo: &Path, managed: &Path) -> Result<()> {
    let relative = utils::relative(repo, managed);

    if !files::exists("tmpl new", managed)? {
        output::warn(format!(
            "the configuration leaves {} out of the repository",
            relative.display()
        ));
        return Ok(());
    }

    output::note(format!("added {}", relative.display()));
    Ok(())
}

fn absolute(home: &Path, arg: &str) -> Result<PathBuf> {
    let path = Path::new(arg);
    if path.starts_with("~") {
        return Ok(utils::expand(home, path));
    }

    std::path::absolute(path).with_context(|| format!("tmpl new: invalid path {arg}"))
}

fn create(template: &Path, managed: &Path, file: bool) -> Result<()> {
    check(template, managed)?;

    match file {
        true => create_file(template),
        false => create_dir(template),
    }
}

fn check(template: &Path, managed: &Path) -> Result<()> {
    if std::fs::symlink_metadata(template).is_ok() {
        bail!("tmpl new: {} already exists", template.display());
    }
    if std::fs::symlink_metadata(managed).is_ok() {
        bail!(
            "tmpl new: {} already exists in the repository",
            managed.display()
        );
    }
    if let Some(target) = files::template_target(managed)
        && std::fs::symlink_metadata(&target).is_ok()
    {
        bail!(
            "tmpl new: {} is already in the repository; run `luadot rm` on it first",
            target.display()
        );
    }

    Ok(())
}

fn create_file(template: &Path) -> Result<()> {
    create_parent(template)?;

    std::fs::write(template, "")
        .with_context(|| format!("tmpl new: failed to create {}", template.display()))
}

fn create_dir(template: &Path) -> Result<()> {
    std::fs::create_dir_all(template)
        .with_context(|| format!("tmpl new: failed to create {}", template.display()))?;

    let script = template.join(TEMPLATE_FILE);
    std::fs::write(&script, TEMPLATE_SKELETON)
        .with_context(|| format!("tmpl new: failed to create {}", script.display()))
}

fn create_parent(template: &Path) -> Result<()> {
    let Some(parent) = template.parent() else {
        return Ok(());
    };

    std::fs::create_dir_all(parent)
        .with_context(|| format!("tmpl new: failed to create {}", parent.display()))
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
        let (_root, home, _repo) = home_and_repo();

        assert_eq!(
            destination(&home, &arg(&home.join(".zshrc"))).unwrap(),
            home.join(".zshrc.luadot")
        );
    }

    #[test]
    fn a_nested_path_stays_next_to_the_file_it_produces() {
        let (_root, home, _repo) = home_and_repo();

        assert_eq!(
            destination(&home, &arg(&home.join(".config/nvim/init.lua"))).unwrap(),
            home.join(".config/nvim/init.lua.luadot")
        );
    }

    #[test]
    fn a_system_path_keeps_its_own_location() {
        let (_root, home, _repo) = home_and_repo();

        assert_eq!(
            destination(&home, "/etc/zsh/zshrc").unwrap(),
            PathBuf::from("/etc/zsh/zshrc.luadot")
        );
    }

    #[test]
    fn an_existing_template_is_left_alone() {
        let (_root, home, repo) = home_and_repo();
        let template = home.join(".zshrc.luadot");
        std::fs::create_dir_all(&template).unwrap();
        std::fs::write(template.join(TEMPLATE_FILE), "kept").unwrap();

        let err = create(&template, &repo.join("home/.zshrc.luadot"), false)
            .unwrap_err()
            .to_string();

        assert!(err.contains("tmpl new: "));
        assert!(err.contains("already exists"));
        assert_eq!(
            std::fs::read_to_string(template.join(TEMPLATE_FILE)).unwrap(),
            "kept"
        );
    }

    #[test]
    fn a_template_the_repository_already_holds_is_reported() {
        let (_root, home, repo) = home_and_repo();
        let managed = repo.join("home/.zshrc.luadot");
        std::fs::create_dir_all(&managed).unwrap();

        let err = create(&home.join(".zshrc.luadot"), &managed, false)
            .unwrap_err()
            .to_string();

        assert!(err.contains("tmpl new: "));
        assert!(err.contains("already exists in the repository"));
        assert!(!home.join(".zshrc.luadot").exists());
    }

    #[test]
    fn a_file_the_repository_already_manages_is_reported() {
        let (_root, home, repo) = home_and_repo();
        std::fs::create_dir_all(repo.join("home")).unwrap();
        std::fs::write(repo.join("home/.zshrc"), "managed").unwrap();

        let err = create(
            &home.join(".zshrc.luadot"),
            &repo.join("home/.zshrc.luadot"),
            false,
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("tmpl new: "));
        assert!(err.contains("already in the repository"));
    }
}
