use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Paths {
    home: PathBuf,
    config: PathBuf,
    data: PathBuf,
    repo: Option<PathBuf>,
    dir: Option<PathBuf>,
}

impl Paths {
    pub fn new(home: &Path, config: &Path, data: &Path) -> Self {
        Self {
            home: home.to_path_buf(),
            config: config.to_path_buf(),
            data: data.to_path_buf(),
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

    pub fn data(&self) -> &Path {
        &self.data
    }

    pub fn repo(&self) -> Option<&Path> {
        self.repo.as_deref()
    }

    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }
}
