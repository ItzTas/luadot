use std::fs::{File, Metadata, OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
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
    options.write(true).create(true).truncate(true);
    if let Some(mode) = mode {
        options.mode(mode);
    }

    let file = options.open(dest).with_context(failed)?;
    persist(file, contents).with_context(failed)?;

    let Some(mode) = mode else {
        return Ok(());
    };

    std::fs::set_permissions(dest, Permissions::from_mode(mode))
        .with_context(|| format!("{command}: failed to set the mode of {}", dest.display()))
}

fn persist(mut file: File, contents: &[u8]) -> std::io::Result<()> {
    file.write_all(contents)?;

    file.sync_all()
}
