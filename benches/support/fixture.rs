use std::path::{Path, PathBuf};

use luadot::files::{LinkMode, link};
use tempfile::TempDir;

use super::tree::{fill, git_noise, templates, write};

pub struct Fixture {
    home: TempDir,
    repo: TempDir,
    files: Vec<PathBuf>,
}

impl Fixture {
    pub fn new(count: usize) -> Self {
        let home = tempfile::tempdir().expect("a temporary home");
        let repo = tempfile::tempdir().expect("a temporary repository");
        let files = fill(repo.path(), count);
        git_noise(repo.path());

        Self { home, repo, files }
    }

    pub fn with_templates(count: usize) -> Self {
        let fixture = Self::new(count);
        templates(fixture.repo());

        fixture
    }

    pub fn home(&self) -> &Path {
        self.home.path()
    }

    pub fn repo(&self) -> &Path {
        self.repo.path()
    }

    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn relative<'a>(&self, file: &'a Path) -> &'a Path {
        file.strip_prefix(self.repo.path())
            .expect("a file of the repository")
    }

    pub fn destination(&self, file: &Path) -> PathBuf {
        self.home.path().join(self.relative(file))
    }

    pub fn pairs(&self) -> Vec<(PathBuf, PathBuf)> {
        self.files
            .iter()
            .map(|file| (file.clone(), self.destination(file)))
            .collect()
    }

    pub fn spread(&self, mode: LinkMode) {
        for (index, file) in self.files.iter().enumerate() {
            let dest = self.destination(file);
            match index % 4 {
                0 => continue,
                1 => place(mode, file, &dest),
                2 => write(&dest, &contents(file)),
                _ => write(&dest, "diverged\n"),
            }
        }
    }
}

fn place(mode: LinkMode, source: &Path, dest: &Path) {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).expect("a parent directory");
    }
    link(mode, source, dest).expect("a linked file");
}

fn contents(file: &Path) -> String {
    std::fs::read_to_string(file).expect("a readable file")
}
