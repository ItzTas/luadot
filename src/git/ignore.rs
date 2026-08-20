use std::path::Path;

use anyhow::{Context, Result};
use gix::index::entry::Mode;
use gix::worktree::stack::state::ignore::Source;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    File,
    Directory,
}

pub struct Excludes {
    command: String,
    git: Option<Git>,
}

struct Git {
    repository: gix::Repository,
    stack: gix::worktree::Stack,
}

impl Excludes {
    pub fn open(command: &str, repo: &Path) -> Result<Self> {
        let command = command.to_string();

        let Ok(repository) = gix::open(repo) else {
            return Ok(Self { command, git: None });
        };

        let index = repository.index_or_empty().with_context(|| {
            format!("{command}: failed to read the index of {}", repo.display())
        })?;
        let stack = repository
            .excludes(&index, None, Source::default())
            .with_context(|| {
                format!(
                    "{command}: failed to read the .gitignore files of {}",
                    repo.display()
                )
            })?
            .detach();

        Ok(Self {
            command,
            git: Some(Git { repository, stack }),
        })
    }

    pub fn excluded(&mut self, relative: &Path, kind: Kind) -> Result<bool> {
        let Self { command, git } = self;
        let Some(git) = git.as_mut() else {
            return Ok(false);
        };

        let mode = match kind {
            Kind::Directory => Mode::DIR,
            Kind::File => Mode::FILE,
        };
        let platform = git
            .stack
            .at_path(relative, Some(mode), &git.repository)
            .with_context(|| {
                format!(
                    "{command}: failed to match {} against .gitignore",
                    relative.display()
                )
            })?;

        Ok(platform.is_excluded())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn repository(rules: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        gix::init(&repo).unwrap();
        std::fs::write(repo.join(".gitignore"), rules).unwrap();

        (dir, repo)
    }

    #[test]
    fn a_pattern_excludes_the_files_it_names() {
        let (_dir, repo) = repository("*.swp\n");
        let mut excludes = Excludes::open("add", &repo).unwrap();

        assert!(
            excludes
                .excluded(Path::new("home/.vimrc.swp"), Kind::File)
                .unwrap()
        );
        assert!(
            !excludes
                .excluded(Path::new("home/.vimrc"), Kind::File)
                .unwrap()
        );
    }

    #[test]
    fn an_excluded_directory_carries_its_contents() {
        let (_dir, repo) = repository("home/.cache/\n");
        let mut excludes = Excludes::open("add", &repo).unwrap();

        assert!(
            !excludes
                .excluded(Path::new("home/.cache"), Kind::File)
                .unwrap()
        );
        assert!(
            excludes
                .excluded(Path::new("home/.cache"), Kind::Directory)
                .unwrap()
        );
        assert!(
            excludes
                .excluded(Path::new("home/.cache/nvim/log"), Kind::File)
                .unwrap()
        );
    }

    #[test]
    fn a_negated_pattern_takes_a_file_back() {
        let (_dir, repo) = repository("home/.config/*\n!home/.config/nvim\n");
        let mut excludes = Excludes::open("add", &repo).unwrap();

        assert!(
            excludes
                .excluded(Path::new("home/.config/fish"), Kind::Directory)
                .unwrap()
        );
        assert!(
            !excludes
                .excluded(Path::new("home/.config/nvim"), Kind::Directory)
                .unwrap()
        );
    }
}
