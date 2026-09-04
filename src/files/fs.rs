use std::fs::{File, Metadata, OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::constants::MODE_BITS;

pub fn exists(command: &str, path: &Path) -> Result<bool> {
    Ok(metadata(command, path)?.is_some())
}

pub fn metadata(command: &str, path: &Path) -> Result<Option<Metadata>> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) => Ok(Some(meta)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => {
            Err(err).with_context(|| format!("{command}: failed to inspect {}", path.display()))
        }
    }
}

pub fn create_parent(command: &str, path: &Path) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)
        .with_context(|| format!("{command}: failed to create {}", parent.display()))
}

pub fn private_dir(command: &str, dir: &Path, mode: u32) -> Result<()> {
    let failed = || format!("{command}: failed to create {}", dir.display());

    if let Some(above) = dir.parent() {
        std::fs::create_dir_all(above).with_context(failed)?;
    }
    match std::fs::DirBuilder::new().mode(mode).create(dir) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => narrowed(command, dir, mode),
        Err(err) => Err(err).with_context(failed),
    }
}

fn narrowed(command: &str, dir: &Path, mode: u32) -> Result<()> {
    let meta = std::fs::metadata(dir)
        .with_context(|| format!("{command}: failed to inspect {}", dir.display()))?;
    if mode_bits(&meta) == mode {
        return Ok(());
    }

    std::fs::set_permissions(dir, Permissions::from_mode(mode))
        .with_context(|| format!("{command}: failed to set the mode of {}", dir.display()))
}

pub fn prune_parents(command: &str, root: &Path, file: &Path) -> Result<()> {
    let mut current = file.parent();
    while let Some(dir) = current.filter(|dir| *dir != root && dir.starts_with(root)) {
        if !is_empty(command, dir)? {
            return Ok(());
        }
        std::fs::remove_dir(dir)
            .with_context(|| format!("{command}: failed to remove {}", dir.display()))?;
        current = dir.parent();
    }

    Ok(())
}

fn is_empty(command: &str, dir: &Path) -> Result<bool> {
    let mut entries = std::fs::read_dir(dir)
        .with_context(|| format!("{command}: failed to read {}", dir.display()))?;

    Ok(entries.next().is_none())
}

pub fn link_target(command: &str, source: &Path) -> Result<(Metadata, Option<PathBuf>)> {
    let meta = std::fs::symlink_metadata(source)
        .with_context(|| format!("{command}: failed to inspect {}", source.display()))?;
    if !meta.file_type().is_symlink() {
        return Ok((meta, None));
    }

    let target = std::fs::read_link(source)
        .with_context(|| format!("{command}: failed to read {}", source.display()))?;

    Ok((meta, Some(target)))
}

pub fn link_at(command: &str, path: &Path) -> Result<Option<PathBuf>> {
    let Some(meta) = metadata(command, path)? else {
        return Ok(None);
    };
    if !meta.file_type().is_symlink() {
        return Ok(None);
    }

    std::fs::read_link(path)
        .map(Some)
        .with_context(|| format!("{command}: failed to read {}", path.display()))
}

pub fn read_contents(command: &str, path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).with_context(|| format!("{command}: failed to read {}", path.display()))
}

pub fn regular_file(path: &Path) -> Option<Metadata> {
    let meta = std::fs::symlink_metadata(path).ok()?;

    meta.file_type().is_file().then_some(meta)
}

pub fn mode_bits(meta: &Metadata) -> u32 {
    meta.permissions().mode() & MODE_BITS
}

pub fn effective_mode(command: &str, source: &Path, mode: Option<u32>) -> Result<u32> {
    if let Some(mode) = mode {
        return Ok(mode);
    }

    let meta = std::fs::metadata(source)
        .with_context(|| format!("{command}: failed to inspect {}", source.display()))?;
    Ok(mode_bits(&meta))
}

pub fn write_mode(command: &str, dest: &Path, contents: &[u8], mode: u32) -> Result<()> {
    write_contents(command, dest, contents, Some(mode))
}

pub fn write_contents(
    command: &str,
    dest: &Path,
    contents: &[u8],
    mode: Option<u32>,
) -> Result<()> {
    let failed = || format!("{command}: failed to write {}", dest.display());

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    if let Some(mode) = mode {
        options.mode(mode);
    }

    let file = options.open(dest).with_context(failed)?;
    if let Some(mode) = mode {
        file.set_permissions(Permissions::from_mode(mode))
            .with_context(|| format!("{command}: failed to set the mode of {}", dest.display()))?;
    }

    persist(file, contents).with_context(failed)
}

fn persist(mut file: File, contents: &[u8]) -> std::io::Result<()> {
    file.write_all(contents)?;

    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits(path: &Path) -> u32 {
        mode_bits(&std::fs::symlink_metadata(path).unwrap())
    }

    #[test]
    fn a_private_directory_is_narrowed_whatever_it_was() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("share/luadot");

        private_dir("state", &dir, 0o700).unwrap();
        assert_eq!(bits(&dir), 0o700);

        std::fs::set_permissions(&dir, Permissions::from_mode(0o755)).unwrap();
        private_dir("state", &dir, 0o700).unwrap();
        assert_eq!(bits(&dir), 0o700);
    }

    #[test]
    fn a_taken_name_is_never_written_over() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("staged");
        std::fs::write(&dest, "planted").unwrap();

        let err = write_contents("apply", &dest, b"mine", None).unwrap_err();

        assert!(err.to_string().contains("failed to write"));
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "planted");
    }
}
