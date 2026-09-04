use std::path::Path;

use anyhow::{Context, Result, bail};
use tracing::debug;

use super::atomic::replace_file;
use super::constants::COMMAND;
use super::fs::{exists, link_target, regular_file};
use super::placement::Placement;
use super::{LinkMode, link};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictPolicy {
    #[default]
    Overwrite,
    Skip,
    Error,
}

impl ConflictPolicy {
    pub fn name(self) -> &'static str {
        match self {
            Self::Overwrite => "overwrite",
            Self::Skip => "skip",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncOutcome {
    Created,
    Replaced,
    AlreadySynced,
    Skipped,
}

pub fn sync_file(
    policy: ConflictPolicy,
    placement: Placement,
    source: &Path,
    dest: &Path,
) -> Result<SyncOutcome> {
    let outcome = sync(policy, placement, source, dest)?;
    debug!(
        source = %source.display(),
        dest = %dest.display(),
        outcome = ?outcome,
        "synced"
    );
    Ok(outcome)
}

fn sync(
    policy: ConflictPolicy,
    placement: Placement,
    source: &Path,
    dest: &Path,
) -> Result<SyncOutcome> {
    placeable(placement, source)?;

    if !exists(COMMAND, dest)? {
        place(placement, source, dest)?;
        return Ok(SyncOutcome::Created);
    }

    if already_synced(placement.link(), source, dest)? && placement.carried_by(dest) {
        return Ok(SyncOutcome::AlreadySynced);
    }

    if let Some(outcome) = refused(COMMAND, policy, dest)? {
        return Ok(outcome);
    }

    place(placement, source, dest)?;

    Ok(SyncOutcome::Replaced)
}

pub fn refused(command: &str, policy: ConflictPolicy, dest: &Path) -> Result<Option<SyncOutcome>> {
    match policy {
        ConflictPolicy::Skip => Ok(Some(SyncOutcome::Skipped)),
        ConflictPolicy::Error => bail!("{command}: {} already exists", dest.display()),
        ConflictPolicy::Overwrite => Ok(None),
    }
}

fn placeable(placement: Placement, source: &Path) -> Result<()> {
    let (meta, target) = link_target(COMMAND, source)?;
    if target.is_none() {
        if !meta.file_type().is_file() {
            bail!("files: {} is not a file", source.display());
        }
        return Ok(());
    }

    if placement.mode().is_none() && placement.owner().is_none() {
        return Ok(());
    }

    bail!(
        "files: {} is a symlink, so a `mode` or an `owner` would land on what it points at",
        source.display()
    )
}

fn place(placement: Placement, source: &Path, dest: &Path) -> Result<()> {
    replace_file(COMMAND, dest, |staged| {
        link(placement.link(), source, staged)?;

        placement.set_on(COMMAND, attributed(placement.link(), source, staged))
    })
}

fn attributed<'a>(mode: LinkMode, source: &'a Path, staged: &'a Path) -> &'a Path {
    match mode {
        LinkMode::Symbolic => source,
        LinkMode::Hard | LinkMode::Copy => staged,
    }
}

pub(super) fn already_synced(mode: LinkMode, source: &Path, dest: &Path) -> Result<bool> {
    match mode {
        LinkMode::Hard => hard_linked(source, dest),
        LinkMode::Symbolic => points_to(dest, source),
        LinkMode::Copy => copied(source, dest),
    }
}

fn hard_linked(source: &Path, dest: &Path) -> Result<bool> {
    if same_file(source, dest) {
        return Ok(true);
    }
    if same_device(source, dest) {
        return Ok(false);
    }

    copied(source, dest)
}

fn copied(source: &Path, dest: &Path) -> Result<bool> {
    if regular_file(dest).is_none() || same_file(source, dest) {
        return Ok(false);
    }

    same_contents(source, dest)
}

pub(super) fn same_contents(source: &Path, dest: &Path) -> Result<bool> {
    let expected = std::fs::read(source)
        .with_context(|| format!("files: failed to read {}", source.display()))?;

    Ok(matches!(std::fs::read(dest), Ok(found) if found == expected))
}

fn points_to(dest: &Path, source: &Path) -> Result<bool> {
    let Ok(meta) = std::fs::symlink_metadata(dest) else {
        return Ok(false);
    };
    if !meta.file_type().is_symlink() {
        return Ok(false);
    }
    Ok(std::fs::read_link(dest)? == source)
}

fn same_file(source: &Path, dest: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;

    let (Ok(a), Ok(b)) = (std::fs::metadata(source), std::fs::metadata(dest)) else {
        return false;
    };
    a.dev() == b.dev() && a.ino() == b.ino()
}

fn same_device(source: &Path, dest: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;

    let (Ok(a), Ok(b)) = (std::fs::metadata(source), std::fs::metadata(dest)) else {
        return true;
    };

    a.dev() == b.dev()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::MetadataExt;

    fn write(path: &Path, contents: &str) {
        std::fs::write(path, contents).unwrap();
    }

    fn placed(link: LinkMode) -> Placement<'static> {
        Placement::new(link)
    }

    fn other_device() -> Option<tempfile::TempDir> {
        let here = std::fs::metadata(std::env::temp_dir()).ok()?.dev();
        let there = tempfile::tempdir_in("/dev/shm").ok()?;

        match std::fs::metadata(there.path()).ok()?.dev() == here {
            true => None,
            false => Some(there),
        }
    }

    #[test]
    fn a_refused_hard_link_reads_as_synced() {
        let Some(elsewhere) = other_device() else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = elsewhere.path().join("dest");
        write(&source, "data");

        let created = sync_file(
            ConflictPolicy::Overwrite,
            placed(LinkMode::Hard),
            &source,
            &dest,
        )
        .unwrap();

        assert_eq!(created, SyncOutcome::Created);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
        assert!(!same_file(&source, &dest));

        let again = sync_file(
            ConflictPolicy::Overwrite,
            placed(LinkMode::Hard),
            &source,
            &dest,
        )
        .unwrap();

        assert_eq!(again, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn creates_a_hard_link() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");

        let outcome = sync_file(
            ConflictPolicy::Overwrite,
            placed(LinkMode::Hard),
            &source,
            &dest,
        )
        .unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "data");
        assert_eq!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );
    }

    #[test]
    fn symbolic_mode_creates_a_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "data");

        let outcome = sync_file(
            ConflictPolicy::Overwrite,
            placed(LinkMode::Symbolic),
            &source,
            &dest,
        )
        .unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert!(
            std::fs::symlink_metadata(&dest)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read_link(&dest).unwrap(), source);

        let outcome = sync_file(
            ConflictPolicy::Overwrite,
            placed(LinkMode::Symbolic),
            &source,
            &dest,
        )
        .unwrap();
        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn a_symlink_entry_refuses_a_mode() {
        let dir = tempfile::tempdir().unwrap();
        let outside = dir.path().join("outside");
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&outside, "root only");
        std::os::unix::fs::symlink(&outside, &source).unwrap();

        let err = sync_file(
            ConflictPolicy::Overwrite,
            placed(LinkMode::Hard).with_mode(Some(0o777)),
            &source,
            &dest,
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            err,
            format!(
                "files: {} is a symlink, so a `mode` or an `owner` would land on what it points at",
                source.display()
            )
        );
        assert!(!dest.exists());
    }

    #[test]
    fn a_mode_is_restored_on_drift() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "secret");
        let placement = placed(LinkMode::Copy).with_mode(Some(0o600));

        let outcome = sync_file(ConflictPolicy::Overwrite, placement, &source, &dest).unwrap();
        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(
            std::fs::metadata(&dest).unwrap().permissions().mode() & 0o7777,
            0o600
        );

        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o644)).unwrap();
        let outcome = sync_file(ConflictPolicy::Overwrite, placement, &source, &dest).unwrap();
        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(
            std::fs::metadata(&dest).unwrap().permissions().mode() & 0o7777,
            0o600
        );
    }
}
