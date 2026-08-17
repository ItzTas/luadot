use std::fs::{Metadata, OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::constants::MODE_BITS;

pub fn exists(command: &str, path: &Path) -> Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
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

pub fn remove_existing(command: &str, path: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)
        .with_context(|| format!("{command}: failed to inspect {}", path.display()))?;
    if meta.file_type().is_dir() {
        bail!(
            "{command}: refusing to replace directory {} with a file",
            path.display()
        );
    }
    std::fs::remove_file(path)
        .with_context(|| format!("{command}: failed to remove {}", path.display()))
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

pub fn regular_file(path: &Path) -> Option<Metadata> {
    let meta = std::fs::symlink_metadata(path).ok()?;

    meta.file_type().is_file().then_some(meta)
}

pub fn mode_bits(meta: &Metadata) -> u32 {
    meta.permissions().mode() & MODE_BITS
}

pub fn write_mode(command: &str, dest: &Path, contents: &[u8], mode: u32) -> Result<()> {
    let failed = || format!("{command}: failed to write {}", dest.display());

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(dest)
        .with_context(failed)?
        .write_all(contents)
        .with_context(failed)?;

    std::fs::set_permissions(dest, Permissions::from_mode(mode))
        .with_context(|| format!("{command}: failed to set the mode of {}", dest.display()))
}
