use std::path::{Path, PathBuf};

use super::paths::relative;
use crate::lua::Config;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Automatic {
    pub commits: bool,
    pub pushes: bool,
}

pub fn automatic(config: &Config, repo: &Path, paths: &[PathBuf]) -> Automatic {
    Automatic {
        commits: paths
            .iter()
            .any(|path| config.autocommit(relative(repo, path))),
        pushes: paths
            .iter()
            .any(|path| config.autopush(relative(repo, path))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::from_source;

    fn config(source: &str) -> Config {
        from_source(source).unwrap()
    }

    #[test]
    fn one_path_asking_for_it_is_enough() {
        let repo = Path::new("/repo");
        let paths = vec![
            PathBuf::from("/repo/.bashrc"),
            PathBuf::from("/repo/.ssh/config"),
        ];

        let automatic = automatic(
            &config(r#"ld.rules({ match = ".ssh/**", autocommit = true })"#),
            repo,
            &paths,
        );

        assert_eq!(
            automatic,
            Automatic {
                commits: true,
                pushes: false,
            }
        );
    }
}
