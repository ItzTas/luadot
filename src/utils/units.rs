use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use super::paths::relative;
use crate::crypt;
use crate::files::{self, Entry, LinkMode};
use crate::lua::Config;

#[derive(Debug, PartialEq, Eq)]
pub enum Managed {
    File(PathBuf),
    Unit(Unit),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Unit {
    root: PathBuf,
}

impl Unit {
    pub fn root(&self) -> &Path {
        &self.root
    }
}

pub fn units(
    command: &str,
    config: &Config,
    repo: &Path,
    files: Vec<PathBuf>,
) -> Result<Vec<Managed>> {
    let mut managed = Vec::new();
    let mut covered: Option<PathBuf> = None;
    for file in files {
        if covered.as_ref().is_some_and(|root| file.starts_with(root)) {
            continue;
        }

        let logical = crypt::logical(relative(repo, &file));
        let root = config
            .unit_root(&logical)
            .map(|root| repo.join(root))
            .filter(|root| root.is_dir());
        let Some(root) = root else {
            managed.push(Managed::File(file));
            continue;
        };

        managed.push(Managed::Unit(gathered(command, config, repo, &root)?));
        covered = Some(root);
    }

    Ok(managed)
}

pub fn whole_link(command: &str, config: &Config, relative: &Path) -> Result<LinkMode> {
    let link = config.link_mode(relative);
    if link != LinkMode::Hard {
        return Ok(link);
    }

    bail!(
        "{command}: {} is placed whole, which takes link \"{}\" or \"{}\"",
        relative.display(),
        LinkMode::Symbolic.name(),
        LinkMode::Copy.name()
    )
}

fn gathered(command: &str, config: &Config, repo: &Path, root: &Path) -> Result<Unit> {
    let rel = relative(repo, root);
    whole_link(command, config, rel)?;

    for entry in files::collect_entries(command, root)? {
        let inner = relative(repo, entry.path());
        match entry {
            Entry::File(_) => {
                if crypt::split(inner).is_some() {
                    bail!(
                        "{command}: {} is placed whole and cannot hold the encrypted {}",
                        rel.display(),
                        crypt::logical(inner).display()
                    );
                }
                if config.is_ignored(inner) {
                    bail!(
                        "{command}: {} is placed whole, but the rules leave {} out",
                        rel.display(),
                        inner.display()
                    );
                }
            }
            Entry::Template(_) | Entry::Standalone(_) => {
                bail!(
                    "{command}: {} is placed whole and cannot hold the template {}",
                    rel.display(),
                    inner.display()
                );
            }
        }
    }

    Ok(Unit {
        root: root.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::from_source;

    fn write(path: &Path, contents: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    fn nvim_repo(root: &Path) -> (PathBuf, Vec<PathBuf>) {
        let repo = root.join("repo");
        let init = repo.join(".config/nvim/init.lua");
        let plugins = repo.join(".config/nvim/lua/plugins.lua");
        let vimrc = repo.join(".vimrc");
        write(&init, "init");
        write(&plugins, "plugins");
        write(&vimrc, "vim");

        (repo, vec![init, plugins, vimrc])
    }

    #[test]
    fn a_whole_directory_groups_its_files_into_one_unit() {
        let dir = tempfile::tempdir().unwrap();
        let (repo, files) = nvim_repo(dir.path());
        let config =
            from_source(r#"ld.rules({ match = ".config/nvim", whole = true, link = "symbolic" })"#)
                .unwrap();

        let managed = units("apply", &config, &repo, files).unwrap();

        assert_eq!(managed.len(), 2);
        let Managed::Unit(unit) = &managed[0] else {
            panic!("expected a unit first");
        };
        assert_eq!(unit.root(), repo.join(".config/nvim"));
        assert_eq!(managed[1], Managed::File(repo.join(".vimrc")));
    }

    #[test]
    fn a_whole_directory_refuses_a_hard_link() {
        let dir = tempfile::tempdir().unwrap();
        let (repo, files) = nvim_repo(dir.path());
        let config = from_source(r#"ld.rules({ match = ".config/nvim", whole = true })"#).unwrap();

        let err = units("apply", &config, &repo, files)
            .unwrap_err()
            .to_string();

        assert_eq!(
            err,
            "apply: .config/nvim is placed whole, which takes link \"symbolic\" or \"copy\""
        );
    }

    #[test]
    fn a_whole_directory_refuses_an_excluded_file() {
        let dir = tempfile::tempdir().unwrap();
        let (repo, files) = nvim_repo(dir.path());
        let config = from_source(
            r#"
            ld.rules({
              { match = ".config/nvim", whole = true, link = "symbolic" },
              { match = "**/*.lua", track = "never" },
            })
            "#,
        )
        .unwrap();

        let err = units("apply", &config, &repo, files)
            .unwrap_err()
            .to_string();

        assert_eq!(
            err,
            "apply: .config/nvim is placed whole, but the rules leave .config/nvim/init.lua out"
        );
    }

    #[test]
    fn a_whole_rule_matching_a_file_leaves_it_alone() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        let vimrc = repo.join(".vimrc");
        write(&vimrc, "vim");
        let config =
            from_source(r#"ld.rules({ match = ".vimrc", whole = true, link = "symbolic" })"#)
                .unwrap();

        let managed = units("apply", &config, &repo, vec![vimrc.clone()]).unwrap();

        assert_eq!(managed, vec![Managed::File(vimrc)]);
    }
}
