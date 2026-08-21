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
    fn stored_appends_the_backend_extension() {
        assert_eq!(
            stored(Path::new(".netrc"), Backend::Age),
            PathBuf::from(".netrc.age")
        );
        assert_eq!(
            stored(Path::new(".ssh/id_ed25519"), Backend::Gpg),
            PathBuf::from(".ssh/id_ed25519.gpg")
        );
    }

    #[test]
    fn split_reads_the_extension_back() {
        assert_eq!(
            split(Path::new(".netrc.age")),
            Some((PathBuf::from(".netrc"), Backend::Age))
        );
        assert_eq!(
            split(Path::new(".ssh/id_ed25519.gpg")),
            Some((PathBuf::from(".ssh/id_ed25519"), Backend::Gpg))
        );
    }

    #[test]
    fn split_leaves_plain_files_alone() {
        assert_eq!(split(Path::new(".netrc")), None);
        assert_eq!(split(Path::new(".config/nvim/init.lua")), None);
        assert_eq!(split(Path::new(".age")), None);
        assert_eq!(split(Path::new("agenda")), None);
    }

    #[test]
    fn logical_strips_only_what_split_recognizes() {
        assert_eq!(logical(Path::new(".netrc.age")), PathBuf::from(".netrc"));
        assert_eq!(logical(Path::new(".netrc")), PathBuf::from(".netrc"));
    }

    #[test]
    fn stored_variant_finds_the_file_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join(".netrc");
        std::fs::write(stored(&target, Backend::Gpg), "cipher").unwrap();

        assert_eq!(stored_variant(&target), Some(stored(&target, Backend::Gpg)));
        assert_eq!(stored_variant(&dir.path().join(".vimrc")), None);
    }
}
