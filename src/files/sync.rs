use std::path::Path;

use anyhow::{Context, Result, bail};

use super::{LinkMode, link};

/// What to do when a destination already exists and differs from the repo file.
///
/// Only [`ConflictPolicy::Overwrite`] is used for now; the other policies are
/// kept ready so a future Lua configuration can choose per sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictPolicy {
    /// Replace the existing file with the repository's version.
    #[default]
    Overwrite,
    /// Leave the existing file untouched.
    #[allow(dead_code)]
    Skip,
    /// Abort with an error.
    #[allow(dead_code)]
    Error,
}

/// Outcome of syncing a single file out to the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncOutcome {
    /// The destination did not exist and was created.
    Created,
    /// An existing, differing destination was replaced.
    Replaced,
    /// The destination already matched the repository file.
    AlreadySynced,
    /// The destination existed and was left untouched.
    Skipped,
}

/// Places `source` (a repository file) at `dest` on the system.
///
/// Hard links cannot cross filesystems, so [`LinkMode::Hard`] falls back to a
/// plain copy when `source` and `dest` live on different devices.
pub fn sync_file(
    policy: ConflictPolicy,
    mode: LinkMode,
    source: &Path,
    dest: &Path,
) -> Result<SyncOutcome> {
    if !source.is_file() {
        bail!("files: {} is not a file", source.display());
    }

    let replacing = match std::fs::symlink_metadata(dest) {
        Ok(_) => {
            if already_synced(mode, source, dest)? {
                return Ok(SyncOutcome::AlreadySynced);
            }
            match policy {
                ConflictPolicy::Overwrite => {
                    remove_existing(dest)?;
                    true
                }
                ConflictPolicy::Skip => return Ok(SyncOutcome::Skipped),
                ConflictPolicy::Error => bail!("files: {} already exists", dest.display()),
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => false,
        Err(err) => {
            return Err(err).with_context(|| format!("files: failed to inspect {}", dest.display()));
        }
    };

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("files: failed to create {}", parent.display()))?;
    }

    place(mode, source, dest)?;

    Ok(if replacing {
        SyncOutcome::Replaced
    } else {
        SyncOutcome::Created
    })
}

fn place(mode: LinkMode, source: &Path, dest: &Path) -> Result<()> {
    match mode {
        LinkMode::Hard => hard_or_copy(source, dest),
        LinkMode::Symbolic => link(mode, source, dest),
    }
}

fn hard_or_copy(source: &Path, dest: &Path) -> Result<()> {
    match std::fs::hard_link(source, dest) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::CrossesDevices => copy(source, dest),
        Err(err) => Err(err).with_context(|| {
            format!(
                "files: failed to hard link {} -> {}",
                dest.display(),
                source.display()
            )
        }),
    }
}

fn copy(source: &Path, dest: &Path) -> Result<()> {
    std::fs::copy(source, dest).map(|_| ()).with_context(|| {
        format!(
            "files: failed to copy {} -> {}",
            source.display(),
            dest.display()
        )
    })
}

fn already_synced(mode: LinkMode, source: &Path, dest: &Path) -> Result<bool> {
    match mode {
        LinkMode::Hard => Ok(same_file(source, dest)),
        LinkMode::Symbolic => match std::fs::symlink_metadata(dest) {
            Ok(meta) if meta.file_type().is_symlink() => Ok(std::fs::read_link(dest)? == source),
            _ => Ok(false),
        },
    }
}

/// Reports whether `source` and `dest` are the same inode (i.e. hard linked).
fn same_file(source: &Path, dest: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;

    let (Ok(a), Ok(b)) = (std::fs::metadata(source), std::fs::metadata(dest)) else {
        return false;
    };
    a.dev() == b.dev() && a.ino() == b.ino()
}

fn remove_existing(path: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)
        .with_context(|| format!("files: failed to inspect {}", path.display()))?;
    if meta.file_type().is_dir() {
        bail!(
            "files: refusing to replace directory {} with a file",
            path.display()
        );
    }
    std::fs::remove_file(path).with_context(|| format!("files: failed to remove {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::MetadataExt;

    fn write(path: &Path, contents: &str) {
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn default_policy_is_overwrite() {
        assert_eq!(ConflictPolicy::default(), ConflictPolicy::Overwrite);
    }

    #[test]
    fn creates_a_hard_link_when_destination_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
        assert_eq!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );
    }

    #[test]
    fn creates_missing_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("nested/deep/dest");
        write(&source, "data");

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
    }

    #[test]
    fn reports_already_synced_when_hard_linked() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");
        sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap();

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn overwrite_replaces_a_differing_destination() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "repo");
        write(&dest, "stale");

        let outcome = sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "repo");
        assert_eq!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );
    }

    #[test]
    fn skip_leaves_an_existing_destination_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "repo");
        write(&dest, "stale");

        let outcome = sync_file(ConflictPolicy::Skip, LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Skipped);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "stale");
    }

    #[test]
    fn error_policy_aborts_on_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "repo");
        write(&dest, "stale");

        assert!(sync_file(ConflictPolicy::Error, LinkMode::Hard, &source, &dest).is_err());
    }

    #[test]
    fn refuses_to_replace_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "repo");
        std::fs::create_dir(&dest).unwrap();

        let err =
            sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap_err();
        assert!(err.to_string().contains("refusing to replace directory"));
    }

    #[test]
    fn symbolic_mode_creates_a_symlink_and_detects_it() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");

        let outcome =
            sync_file(ConflictPolicy::Overwrite, LinkMode::Symbolic, &source, &dest).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert!(
            std::fs::symlink_metadata(&dest)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read_link(&dest).unwrap(), source);

        let outcome =
            sync_file(ConflictPolicy::Overwrite, LinkMode::Symbolic, &source, &dest).unwrap();
        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn errors_when_source_is_not_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("missing");
        let dest = dir.path().join("dest");

        let err =
            sync_file(ConflictPolicy::Overwrite, LinkMode::Hard, &source, &dest).unwrap_err();
        assert!(err.to_string().contains("is not a file"));
    }

    #[test]
    fn copy_duplicates_file_contents_into_a_new_inode() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");

        copy(&source, &dest).unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
        assert_ne!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );
    }
}
