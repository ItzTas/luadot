use std::path::{Path, PathBuf};

use anyhow::Result;

use super::paths::{relative, repo_path};
use crate::crypt;
use crate::files;
use crate::git;
use crate::lua::{Config, Track};

pub fn adoptable(
    command: &str,
    home: &Path,
    repo: &Path,
    config: &Config,
    excludes: &mut git::Excludes,
) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for root in config.adoption_roots() {
        let source = home.join(root);
        if !source.exists() {
            continue;
        }

        for file in files::collect_files(command, &source)? {
            if !adopted(home, repo, config, excludes, &file)? {
                continue;
            }
            found.push(file);
        }
    }
    found.sort();

    Ok(found)
}

fn adopted(
    home: &Path,
    repo: &Path,
    config: &Config,
    excludes: &mut git::Excludes,
    source: &Path,
) -> Result<bool> {
    let dest = repo_path(home, repo, source)?;
    let inside = relative(repo, &dest);

    if config.track(inside) != Track::Auto {
        return Ok(false);
    }
    if held(&dest) || generated(&dest) {
        return Ok(false);
    }

    excludes
        .excluded(inside, git::Kind::File)
        .map(|excluded| !excluded)
}

fn held(dest: &Path) -> bool {
    std::fs::symlink_metadata(dest).is_ok() || crypt::stored_variant(dest).is_some()
}

fn generated(dest: &Path) -> bool {
    files::template_dir(dest).is_some_and(|template| std::fs::symlink_metadata(template).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua;

    fn workspace() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(home.join(".config/nvim")).unwrap();
        std::fs::create_dir_all(&repo).unwrap();

        (dir, home, repo)
    }

    #[test]
    fn an_auto_rule_reaches_the_files_under_it() {
        let (_dir, home, repo) = workspace();
        let init = home.join(".config/nvim/init.lua");
        std::fs::write(&init, "init").unwrap();
        std::fs::write(home.join(".bashrc"), "bash").unwrap();

        let config =
            lua::from_source(r#"ld.rules({ match = ".config/nvim/**", track = "auto" })"#).unwrap();
        let mut excludes = git::Excludes::open("add", &repo).unwrap();

        assert_eq!(
            adoptable("add", &home, &repo, &config, &mut excludes).unwrap(),
            vec![init]
        );
    }

    #[test]
    fn a_managed_file_is_left_out() {
        let (_dir, home, repo) = workspace();
        std::fs::create_dir_all(repo.join(".config/nvim")).unwrap();
        let init = home.join(".config/nvim/init.lua");
        let fresh = home.join(".config/nvim/options.lua");
        std::fs::write(&init, "init").unwrap();
        std::fs::write(&fresh, "options").unwrap();
        std::fs::write(repo.join(".config/nvim/init.lua"), "init").unwrap();

        let config =
            lua::from_source(r#"ld.rules({ match = ".config/nvim/**", track = "auto" })"#).unwrap();
        let mut excludes = git::Excludes::open("add", &repo).unwrap();

        assert_eq!(
            adoptable("add", &home, &repo, &config, &mut excludes).unwrap(),
            vec![fresh]
        );
    }

    #[test]
    fn a_later_rule_holds_a_subtree_back() {
        let (_dir, home, repo) = workspace();
        std::fs::create_dir_all(home.join(".config/nvim/spell")).unwrap();
        let init = home.join(".config/nvim/init.lua");
        std::fs::write(&init, "init").unwrap();
        std::fs::write(home.join(".config/nvim/spell/en.add"), "words").unwrap();

        let config = lua::from_source(
            r#"
            ld.rules({
              { match = ".config/nvim/**", track = "auto" },
              { match = ".config/nvim/spell/**", track = "never" },
            })
            "#,
        )
        .unwrap();
        let mut excludes = git::Excludes::open("add", &repo).unwrap();

        assert_eq!(
            adoptable("add", &home, &repo, &config, &mut excludes).unwrap(),
            vec![init]
        );
    }
}
