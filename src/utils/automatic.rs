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
    fn nothing_written_asks_for_nothing() {
        let repo = Path::new("/repo");

        assert_eq!(
            automatic(&config("ld.opt.autopush(true)"), repo, &[]),
            Automatic::default()
        );
    }

    #[test]
    fn one_path_asking_for_it_is_enough() {
        let repo = Path::new("/repo");
        let paths = vec![
            PathBuf::from("/repo/home/.bashrc"),
            PathBuf::from("/repo/home/.ssh/config"),
        ];

        let automatic = automatic(
            &config(r#"ld.rules({ match = "home/.ssh/**", autocommit = true })"#),
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

    #[test]
    fn a_path_the_rules_leave_out_asks_for_nothing() {
        let repo = Path::new("/repo");
        let paths = vec![PathBuf::from("/repo/home/.bashrc")];

        let automatic = automatic(
            &config(r#"ld.rules({ match = "home/.ssh/**", autopush = true })"#),
            repo,
            &paths,
        );

        assert_eq!(automatic, Automatic::default());
    }
}
