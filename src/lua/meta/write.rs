use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::debug;

use super::constants::{DEFINITIONS, DEFINITIONS_DIR, DEFINITIONS_FILE, LUARC_FILE, STAGED_SUFFIX};
use super::luarc::{libraries, merged};

#[derive(Debug, PartialEq, Eq)]
pub enum Placed {
    Written(PathBuf),
    Merged(PathBuf),
    Kept(PathBuf, String),
}

pub fn install(command: &str, data: &Path) -> Result<Placed> {
    let definitions = data.join(DEFINITIONS_DIR).join(DEFINITIONS_FILE);
    write(command, &definitions, DEFINITIONS)?;

    Ok(Placed::Written(definitions))
}

pub fn point(
    command: &str,
    dir: &Path,
    home: &Path,
    data: &Path,
    registered: &[PathBuf],
) -> Result<Placed> {
    settings(
        command,
        &dir.join(LUARC_FILE),
        &libraries(home, data, registered),
    )
}

pub fn refresh(command: &str, data: &Path) -> Result<()> {
    let definitions = data.join(DEFINITIONS_DIR).join(DEFINITIONS_FILE);
    let Some(current) = read(command, &definitions)? else {
        return Ok(());
    };
    if current == DEFINITIONS {
        return Ok(());
    }

    debug!(path = %definitions.display(), "refreshing the definitions");
    replace(command, &definitions, DEFINITIONS)
}

fn settings(command: &str, path: &Path, libraries: &[String]) -> Result<Placed> {
    let existing = read(command, path)?;
    let Ok(text) = merged(existing.as_deref(), libraries) else {
        return Ok(Placed::Kept(path.to_path_buf(), merged(None, libraries)?));
    };
    write(command, path, &text)?;

    Ok(match existing {
        Some(_) => Placed::Merged(path.to_path_buf()),
        None => Placed::Written(path.to_path_buf()),
    })
}

fn read(command: &str, path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => {
            Err(err).with_context(|| format!("{command}: failed to read {}", path.display()))
        }
    }
}

fn write(command: &str, path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("{command}: failed to create {}", parent.display()))?;
    }

    std::fs::write(path, text)
        .with_context(|| format!("{command}: failed to write {}", path.display()))
}

fn replace(command: &str, path: &Path, text: &str) -> Result<()> {
    let staged = path.with_extension(format!("{STAGED_SUFFIX}.{}", std::process::id()));
    std::fs::write(&staged, text)
        .with_context(|| format!("{command}: failed to write {}", staged.display()))?;
    std::fs::rename(&staged, path)
        .with_context(|| format!("{command}: failed to replace {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_settings_file_that_does_not_parse_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let luarc = dir.path().join(LUARC_FILE);
        std::fs::write(&luarc, "{ // a comment\n}\n").unwrap();

        let placed = point("meta", dir.path(), dir.path(), dir.path(), &[]).unwrap();

        assert_eq!(
            std::fs::read_to_string(&luarc).unwrap(),
            "{ // a comment\n}\n"
        );
        assert!(
            matches!(&placed, Placed::Kept(path, wanted) if path == &luarc && wanted.contains(DEFINITIONS_DIR))
        );
    }
}
