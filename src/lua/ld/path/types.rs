use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Paths {
    home: PathBuf,
    config: PathBuf,
    repo: Option<PathBuf>,
    dir: Option<PathBuf>,
}

impl Paths {
    pub fn new(home: &Path, config: &Path) -> Self {
        Self {
            home: home.to_path_buf(),
            config: config.to_path_buf(),
            repo: None,
            dir: None,
        }
    }

    pub fn with_repo(self, repo: Option<&Path>) -> Self {
        Self {
            repo: repo.map(Path::to_path_buf),
            ..self
        }
    }

    pub fn with_dir(self, dir: &Path) -> Self {
        Self {
            dir: Some(dir.to_path_buf()),
            ..self
        }
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn config(&self) -> &Path {
        &self.config
    }

    pub fn repo(&self) -> Option<&Path> {
        self.repo.as_deref()
    }

    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_configuration_run_knows_the_home_and_the_configuration() {
        let paths = Paths::new(Path::new("/home/u"), Path::new("/home/u/.config/luadot"));

        assert_eq!(paths.home(), Path::new("/home/u"));
        assert_eq!(paths.config(), Path::new("/home/u/.config/luadot"));
        assert_eq!(paths.repo(), None);
        assert_eq!(paths.dir(), None);
    }

    #[test]
    fn a_template_run_adds_the_repository_and_its_own_directory() {
        let paths = Paths::new(Path::new("/home/u"), Path::new("/home/u/.config/luadot"))
            .with_repo(Some(Path::new("/data/repo")))
            .with_dir(Path::new("/data/repo/.zshrc.luadot"));

        assert_eq!(paths.repo(), Some(Path::new("/data/repo")));
        assert_eq!(paths.dir(), Some(Path::new("/data/repo/.zshrc.luadot")));
    }
}
