use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result, bail};
use tracing::debug;

use super::constants::{MODE_BITS, SUDO};
use super::link::{LinkMode, link};
use super::status::FileStatus;
use super::sync::{ConflictPolicy, SyncOutcome, create_parent, exists, remove_existing};

static STAGE: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub struct Staged {
    path: PathBuf,
}

struct Owner<'a> {
    user: &'a str,
    group: Option<&'a str>,
}

pub fn inspect_system(source: &Path, dest: &Path, mode: Option<u32>) -> Result<FileStatus> {
    if !exists(dest)? {
        return Ok(FileStatus::Missing);
    }

    match holds_copy(source, dest, mode) {
        Ok(true) => Ok(FileStatus::Synced),
        Ok(false) => Ok(FileStatus::Differs),
        Err(err) if permission_denied(&err) => Ok(FileStatus::Unreadable),
        Err(err) => Err(err),
    }
}

pub fn escalated_status(source: &Path, dest: &Path, mode: Option<u32>) -> Result<FileStatus> {
    let status = inspect_system(source, dest, mode)?;
    if status != FileStatus::Unreadable {
        return Ok(status);
    }

    let expected = std::fs::read(source)
        .with_context(|| format!("files: failed to read {}", source.display()))?;
    let found = escalated_read("files", dest)?;

    Ok(match found == expected {
        true => FileStatus::Synced,
        false => FileStatus::Differs,
    })
}

pub fn sync_system(
    policy: ConflictPolicy,
    source: &Path,
    dest: &Path,
    mode: Option<u32>,
    owner: Option<&str>,
) -> Result<SyncOutcome> {
    if !source.is_file() {
        bail!("files: {} is not a file", source.display());
    }

    let outcome = match escalated_status(source, dest, mode)? {
        FileStatus::Missing => SyncOutcome::Created,
        FileStatus::Synced => return Ok(SyncOutcome::AlreadySynced),
        _ => match policy {
            ConflictPolicy::Skip => return Ok(SyncOutcome::Skipped),
            ConflictPolicy::Error => bail!("files: {} already exists", dest.display()),
            ConflictPolicy::Overwrite => SyncOutcome::Replaced,
        },
    };

    place_system(source, dest, mode, owner)?;
    debug!(
        source = %source.display(),
        dest = %dest.display(),
        outcome = ?outcome,
        "synced system file"
    );

    Ok(outcome)
}

pub fn import_system(source: &Path, dest: &Path) -> Result<()> {
    match link(LinkMode::Copy, source, dest) {
        Err(err) if permission_denied(&err) => import_escalated(source, dest),
        other => other,
    }
}

pub fn escalate_entry(command: &str, source: &Path, dest: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(source)
        .with_context(|| format!("{command}: failed to inspect {}", source.display()))?;

    if meta.file_type().is_symlink() {
        let target = std::fs::read_link(source)
            .with_context(|| format!("{command}: failed to read {}", source.display()))?;
        if let Some(parent) = dest.parent() {
            sudo(command, dest, ["install", "-d", "--"], [parent])?;
        }
        return sudo(
            command,
            dest,
            ["ln", "-sfn", "--"],
            [target.as_path(), dest],
        );
    }

    let mode = meta.permissions().mode() & MODE_BITS;
    sudo(
        command,
        dest,
        ["install", "-D", "-m", &format!("{mode:o}"), "--"],
        [source, dest],
    )
}

pub fn escalated_read(command: &str, path: &Path) -> Result<Vec<u8>> {
    debug!(path = %path.display(), "reading with sudo");
    let output = Command::new(SUDO)
        .args(["cat", "--"])
        .arg(path)
        .output()
        .with_context(|| format!("{command}: failed to run {SUDO} for {}", path.display()))?;

    if !output.status.success() {
        bail!(
            "{command}: {SUDO} could not read {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(output.stdout)
}

pub fn permission_denied(err: &anyhow::Error) -> bool {
    err.downcast_ref::<std::io::Error>()
        .is_some_and(|io| io.kind() == std::io::ErrorKind::PermissionDenied)
}

pub fn stage_text(text: &str) -> Result<Staged> {
    let dir = std::env::temp_dir();
    loop {
        let name = format!(
            "luadot-stage-{}-{}",
            std::process::id(),
            STAGE.fetch_add(1, Ordering::Relaxed)
        );
        let path = dir.join(name);

        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path);
        let mut file = match file {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("files: failed to stage {}", path.display()));
            }
        };

        let staged = Staged { path };
        file.write_all(text.as_bytes())
            .with_context(|| format!("files: failed to stage {}", staged.path.display()))?;
        return Ok(staged);
    }
}

impl Staged {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Staged {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

impl<'a> Owner<'a> {
    fn parse(raw: &'a str) -> Self {
        match raw.split_once(':') {
            Some((user, group)) => Self {
                user,
                group: Some(group),
            },
            None => Self {
                user: raw,
                group: None,
            },
        }
    }

    fn ids(&self) -> Option<(u32, Option<u32>)> {
        let user = numeric(self.user)?;
        match self.group {
            None => Some((user, None)),
            Some(group) => Some((user, Some(numeric(group)?))),
        }
    }
}

fn numeric(name: &str) -> Option<u32> {
    if name == "root" {
        return Some(0);
    }
    name.parse().ok()
}

fn import_escalated(source: &Path, dest: &Path) -> Result<()> {
    let bytes = escalated_read("files", source)?;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dest)
        .with_context(|| format!("files: failed to write {}", dest.display()))?;
    file.write_all(&bytes)
        .with_context(|| format!("files: failed to write {}", dest.display()))?;

    let mode = std::fs::metadata(source)
        .with_context(|| format!("files: failed to inspect {}", source.display()))?
        .permissions()
        .mode()
        & MODE_BITS;
    std::fs::set_permissions(dest, std::fs::Permissions::from_mode(mode))
        .with_context(|| format!("files: failed to set the mode of {}", dest.display()))
}

fn holds_copy(source: &Path, dest: &Path, mode: Option<u32>) -> Result<bool> {
    let Ok(meta) = std::fs::symlink_metadata(dest) else {
        return Ok(false);
    };
    if !meta.file_type().is_file() {
        return Ok(false);
    }
    if meta.permissions().mode() & MODE_BITS != effective_mode(source, mode)? {
        return Ok(false);
    }

    let found =
        std::fs::read(dest).with_context(|| format!("files: failed to read {}", dest.display()))?;
    let expected = std::fs::read(source)
        .with_context(|| format!("files: failed to read {}", source.display()))?;
    Ok(found == expected)
}

fn effective_mode(source: &Path, mode: Option<u32>) -> Result<u32> {
    if let Some(mode) = mode {
        return Ok(mode);
    }

    let meta = std::fs::metadata(source)
        .with_context(|| format!("files: failed to inspect {}", source.display()))?;
    Ok(meta.permissions().mode() & MODE_BITS)
}

fn place_system(source: &Path, dest: &Path, mode: Option<u32>, owner: Option<&str>) -> Result<()> {
    let parsed = owner.map(Owner::parse);
    if parsed.as_ref().is_some_and(|owner| owner.ids().is_none()) {
        return place_sudo(source, dest, mode, parsed.as_ref());
    }

    match place_plain(source, dest, mode, parsed.as_ref()) {
        Err(err) if permission_denied(&err) => place_sudo(source, dest, mode, parsed.as_ref()),
        other => other,
    }
}

fn place_plain(source: &Path, dest: &Path, mode: Option<u32>, owner: Option<&Owner>) -> Result<()> {
    create_parent(dest)?;
    if exists(dest)? {
        remove_existing(dest)?;
    }
    link(LinkMode::Copy, source, dest)?;

    let mode = effective_mode(source, mode)?;
    std::fs::set_permissions(dest, std::fs::Permissions::from_mode(mode))
        .with_context(|| format!("files: failed to set the mode of {}", dest.display()))?;

    let Some((user, group)) = owner.and_then(Owner::ids) else {
        return Ok(());
    };
    std::os::unix::fs::chown(dest, Some(user), group)
        .with_context(|| format!("files: failed to set the owner of {}", dest.display()))
}

fn place_sudo(source: &Path, dest: &Path, mode: Option<u32>, owner: Option<&Owner>) -> Result<()> {
    let mode = effective_mode(source, mode)?;
    let mut arguments = vec![
        "install".to_string(),
        "-D".to_string(),
        "-m".to_string(),
        format!("{mode:o}"),
    ];
    if let Some(owner) = owner {
        arguments.extend(["-o".to_string(), owner.user.to_string()]);
        if let Some(group) = owner.group {
            arguments.extend(["-g".to_string(), group.to_string()]);
        }
    }
    arguments.push("--".to_string());

    sudo("files", dest, arguments, [source, dest])
}

fn sudo<A, P>(command: &str, dest: &Path, arguments: A, paths: P) -> Result<()>
where
    A: IntoIterator,
    A::Item: AsRef<std::ffi::OsStr>,
    P: IntoIterator,
    P::Item: AsRef<std::ffi::OsStr>,
{
    let mut invocation = Command::new(SUDO);
    invocation.args(arguments).args(paths);
    debug!(?invocation, "escalating");

    let output = invocation
        .output()
        .with_context(|| format!("{command}: failed to run {SUDO} for {}", dest.display()))?;
    if output.status.success() {
        return Ok(());
    }

    bail!(
        "{command}: {SUDO} could not place {}: {}",
        dest.display(),
        String::from_utf8_lossy(&output.stderr).trim()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::MetadataExt;

    fn write(path: &Path, contents: &str) {
        std::fs::write(path, contents).unwrap();
    }

    fn bits(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & MODE_BITS
    }

    #[test]
    fn a_missing_destination_is_created_with_the_mode() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("etc/app.conf");
        write(&source, "conf");

        let outcome =
            sync_system(ConflictPolicy::Overwrite, &source, &dest, Some(0o640), None).unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "conf");
        assert_eq!(bits(&dest), 0o640);
        assert_ne!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );
    }

    #[test]
    fn a_placed_file_is_left_alone_afterwards() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("etc/app.conf");
        write(&source, "conf");

        sync_system(ConflictPolicy::Overwrite, &source, &dest, None, None).unwrap();
        let outcome = sync_system(ConflictPolicy::Overwrite, &source, &dest, None, None).unwrap();

        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn a_mode_that_drifted_is_written_back() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("app.conf");
        write(&source, "conf");
        sync_system(ConflictPolicy::Overwrite, &source, &dest, Some(0o640), None).unwrap();
        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o600)).unwrap();

        let outcome =
            sync_system(ConflictPolicy::Overwrite, &source, &dest, Some(0o640), None).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(bits(&dest), 0o640);
    }

    #[test]
    fn without_a_mode_the_repository_bits_are_kept() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("app.sh");
        write(&source, "#!/bin/sh\n");
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o755)).unwrap();

        sync_system(ConflictPolicy::Overwrite, &source, &dest, None, None).unwrap();

        assert_eq!(bits(&dest), 0o755);
    }

    #[test]
    fn the_policy_gates_a_diverging_destination() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("app.conf");
        write(&source, "repo");
        write(&dest, "system");

        let outcome = sync_system(ConflictPolicy::Skip, &source, &dest, None, None).unwrap();
        assert_eq!(outcome, SyncOutcome::Skipped);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "system");

        assert!(sync_system(ConflictPolicy::Error, &source, &dest, None, None).is_err());

        let outcome = sync_system(ConflictPolicy::Overwrite, &source, &dest, None, None).unwrap();
        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "repo");
    }

    #[test]
    fn a_numeric_owner_naming_the_caller_is_applied_in_place() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("app.conf");
        write(&source, "conf");
        let meta = std::fs::metadata(&source).unwrap();
        let owner = format!("{}:{}", meta.uid(), meta.gid());

        let outcome = sync_system(
            ConflictPolicy::Overwrite,
            &source,
            &dest,
            None,
            Some(&owner),
        )
        .unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::metadata(&dest).unwrap().uid(), meta.uid());
    }

    #[test]
    fn inspect_system_reports_the_content_and_mode_states() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("app.conf");
        write(&source, "conf");

        assert_eq!(
            inspect_system(&source, &dest, None).unwrap(),
            FileStatus::Missing
        );

        sync_system(ConflictPolicy::Overwrite, &source, &dest, Some(0o640), None).unwrap();
        assert_eq!(
            inspect_system(&source, &dest, Some(0o640)).unwrap(),
            FileStatus::Synced
        );
        assert_eq!(
            inspect_system(&source, &dest, Some(0o600)).unwrap(),
            FileStatus::Differs
        );

        write(&dest, "edited");
        assert_eq!(
            inspect_system(&source, &dest, None).unwrap(),
            FileStatus::Differs
        );
    }

    #[test]
    fn a_symlink_never_counts_as_a_placed_copy() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("app.conf");
        write(&source, "conf");
        std::os::unix::fs::symlink(&source, &dest).unwrap();

        assert_eq!(
            inspect_system(&source, &dest, None).unwrap(),
            FileStatus::Differs
        );

        let outcome = sync_system(ConflictPolicy::Overwrite, &source, &dest, None, None).unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert!(
            !std::fs::symlink_metadata(&dest)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn import_system_copies_into_the_repository() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("app.conf");
        let dest = dir.path().join("imported");
        write(&source, "conf");
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o640)).unwrap();

        import_system(&source, &dest).unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "conf");
        assert_eq!(bits(&dest), 0o640);
    }

    #[test]
    fn stage_text_holds_the_text_in_a_private_file_until_dropped() {
        let staged = stage_text("generated\n").unwrap();
        let path = staged.path().to_path_buf();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "generated\n");
        assert_eq!(bits(&path), 0o600);

        drop(staged);
        assert!(!path.exists());
    }

    #[test]
    fn errors_when_the_source_is_not_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("missing");
        let dest = dir.path().join("dest");

        let err = sync_system(ConflictPolicy::Overwrite, &source, &dest, None, None).unwrap_err();
        assert!(err.to_string().contains("is not a file"));
    }

    #[test]
    fn refuses_to_replace_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join("dest");
        write(&source, "conf");
        std::fs::create_dir(&dest).unwrap();

        let err = sync_system(ConflictPolicy::Overwrite, &source, &dest, None, None).unwrap_err();
        assert!(err.to_string().contains("refusing to replace directory"));
    }

    #[test]
    fn an_owner_that_cannot_be_resolved_is_left_to_sudo() {
        let owner = Owner::parse("nobody-here");
        assert!(owner.ids().is_none());

        let owner = Owner::parse("root:root");
        assert_eq!(owner.ids(), Some((0, Some(0))));

        let owner = Owner::parse("1000");
        assert_eq!(owner.ids(), Some((1000, None)));
    }
}
