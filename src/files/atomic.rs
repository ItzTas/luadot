use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result, bail};

use super::constants::{STAGED_ATTEMPTS, STAGED_PREFIX};
use super::fs::{create_parent, metadata, write_contents};

static TICKET: AtomicU64 = AtomicU64::new(0);

pub fn replace_file(
    command: &str,
    dest: &Path,
    place: impl FnOnce(&Path) -> Result<()>,
) -> Result<()> {
    refuse_directory(command, dest)?;
    create_parent(command, dest)?;

    let staged = Staged::beside(command, dest)?;
    place(staged.path())?;

    staged.rename(command, dest)
}

pub fn replace_contents(command: &str, dest: &Path, contents: &[u8]) -> Result<()> {
    replace_file(command, dest, |staged| {
        write_contents(command, staged, contents, None)
    })
}

fn refuse_directory(command: &str, dest: &Path) -> Result<()> {
    let Some(meta) = metadata(command, dest)? else {
        return Ok(());
    };
    if !meta.file_type().is_dir() {
        return Ok(());
    }

    bail!(
        "{command}: refusing to replace directory {} with a file",
        dest.display()
    )
}

struct Staged {
    path: PathBuf,
    parent: PathBuf,
    kept: bool,
}

impl Staged {
    fn beside(command: &str, dest: &Path) -> Result<Self> {
        let Some(name) = dest.file_name() else {
            bail!("{command}: {} does not name a file", dest.display());
        };
        let parent = holding(dest);

        for _ in 0..STAGED_ATTEMPTS {
            let path = parent.join(staged_name(name));
            if metadata(command, &path)?.is_some() {
                continue;
            }

            return Ok(Self {
                path,
                parent,
                kept: false,
            });
        }

        bail!(
            "{command}: failed to reserve a temporary file next to {}",
            dest.display()
        )
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn rename(mut self, command: &str, dest: &Path) -> Result<()> {
        std::fs::rename(&self.path, dest)
            .with_context(|| format!("{command}: failed to replace {}", dest.display()))?;
        self.kept = true;
        flush(&self.parent);

        Ok(())
    }
}

impl Drop for Staged {
    fn drop(&mut self) {
        if self.kept {
            return;
        }

        let _ = std::fs::remove_file(&self.path);
    }
}

fn holding(dest: &Path) -> PathBuf {
    match dest.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

fn staged_name(name: &OsStr) -> OsString {
    let mut staged = OsString::from(".");
    staged.push(name);
    staged.push(format!(
        ".{STAGED_PREFIX}-{}-{}",
        std::process::id(),
        TICKET.fetch_add(1, Ordering::Relaxed)
    ));

    staged
}

fn flush(dir: &Path) {
    let _ = File::open(dir).and_then(|handle| handle.sync_all());
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::*;

    fn entries(dir: &Path) -> Vec<OsString> {
        let mut names: Vec<OsString> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        names.sort();

        names
    }

    #[test]
    fn a_placement_that_fails_leaves_the_destination_and_the_directory_alone() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::write(&dest, "mine").unwrap();

        let err = replace_file("apply", &dest, |staged| {
            std::fs::write(staged, "half").unwrap();
            Err(anyhow!("apply: the hook refused"))
        })
        .unwrap_err()
        .to_string();

        assert_eq!(err, "apply: the hook refused");
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "mine");
        assert_eq!(entries(dir.path()), [OsString::from(".zshrc")]);
    }

    #[test]
    fn a_directory_is_never_replaced_by_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".config");
        std::fs::create_dir(&dest).unwrap();

        let err = replace_file("apply", &dest, |staged| {
            std::fs::write(staged, "generated").map_err(Into::into)
        })
        .unwrap_err()
        .to_string();

        assert!(err.contains("apply: refusing to replace directory"));
        assert!(dest.is_dir());
    }

    #[test]
    fn a_missing_destination_gets_its_parents_before_the_staged_file() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("nested/deeper/.zshrc");

        replace_file("apply", &dest, |staged| {
            std::fs::write(staged, "generated").map_err(Into::into)
        })
        .unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "generated");
        assert_eq!(entries(dest.parent().unwrap()), [OsString::from(".zshrc")]);
    }
}
