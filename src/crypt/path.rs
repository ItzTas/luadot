use std::path::{Path, PathBuf};

use super::backend::Backend;
use super::constants::{AGE, GPG};

pub fn stored(target: &Path, backend: Backend) -> PathBuf {
    let mut path = target.as_os_str().to_os_string();
    path.push(".");
    path.push(backend.extension());
    PathBuf::from(path)
}

pub fn split(relative: &Path) -> Option<(PathBuf, Backend)> {
    let backend = match relative.extension()?.to_str()? {
        AGE => Backend::Age,
        GPG => Backend::Gpg,
        _ => return None,
    };
    Some((relative.with_extension(""), backend))
}

pub fn logical(relative: &Path) -> PathBuf {
    split(relative)
        .map(|(stripped, _)| stripped)
        .unwrap_or_else(|| relative.to_path_buf())
}

pub fn stored_variant(target: &Path) -> Option<PathBuf> {
    [Backend::Age, Backend::Gpg]
        .into_iter()
        .map(|backend| stored(target, backend))
        .find(|candidate| std::fs::symlink_metadata(candidate).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_leaves_plain_files_alone() {
        assert_eq!(split(Path::new(".netrc")), None);
        assert_eq!(split(Path::new(".config/nvim/init.lua")), None);
        assert_eq!(split(Path::new(".age")), None);
        assert_eq!(split(Path::new("agenda")), None);
    }

    #[test]
    fn stored_variant_finds_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join(".netrc");
        std::fs::write(stored(&target, Backend::Gpg), "cipher").unwrap();

        assert_eq!(stored_variant(&target), Some(stored(&target, Backend::Gpg)));
        assert_eq!(stored_variant(&dir.path().join(".vimrc")), None);
    }
}
