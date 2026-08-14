use std::path::Path;

use anyhow::{Context, Result, bail};

use super::sync::{ConflictPolicy, SyncOutcome, create_parent, exists, remove_existing};

pub fn write_file(policy: ConflictPolicy, dest: &Path, contents: &str) -> Result<SyncOutcome> {
    if !exists(dest)? {
        create_parent(dest)?;
        write(dest, contents)?;
        return Ok(SyncOutcome::Created);
    }

    if holds(dest, contents)? {
        return Ok(SyncOutcome::AlreadySynced);
    }

    match policy {
        ConflictPolicy::Skip => return Ok(SyncOutcome::Skipped),
        ConflictPolicy::Error => bail!("files: {} already exists", dest.display()),
        ConflictPolicy::Overwrite => {}
    }

    remove_existing(dest)?;
    write(dest, contents)?;

    Ok(SyncOutcome::Replaced)
}

fn holds(path: &Path, contents: &str) -> Result<bool> {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return Ok(false);
    };
    if !meta.file_type().is_file() {
        return Ok(false);
    }

    Ok(matches!(std::fs::read(path), Ok(found) if found == contents.as_bytes()))
}

fn write(dest: &Path, contents: &str) -> Result<()> {
    std::fs::write(dest, contents)
        .with_context(|| format!("files: failed to write {}", dest.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_a_missing_destination() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("nested/deep/.zshrc");

        let outcome = write_file(ConflictPolicy::Overwrite, &dest, "generated").unwrap();

        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "generated");
    }

    #[test]
    fn matching_contents_are_already_synced() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::write(&dest, "generated").unwrap();

        let outcome = write_file(ConflictPolicy::Error, &dest, "generated").unwrap();

        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn overwrite_replaces_diverging_contents() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::write(&dest, "stale").unwrap();

        let outcome = write_file(ConflictPolicy::Overwrite, &dest, "generated").unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "generated");
    }

    #[test]
    fn skip_leaves_the_destination_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::write(&dest, "stale").unwrap();

        let outcome = write_file(ConflictPolicy::Skip, &dest, "generated").unwrap();

        assert_eq!(outcome, SyncOutcome::Skipped);
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "stale");
    }

    #[test]
    fn error_policy_aborts_on_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::write(&dest, "stale").unwrap();

        assert!(write_file(ConflictPolicy::Error, &dest, "generated").is_err());
    }

    #[test]
    fn a_symlink_never_counts_as_the_generated_file() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let dest = dir.path().join(".zshrc");
        std::fs::write(&source, "generated").unwrap();
        std::os::unix::fs::symlink(&source, &dest).unwrap();

        assert!(!holds(&dest, "generated").unwrap());

        let outcome = write_file(ConflictPolicy::Overwrite, &dest, "generated").unwrap();

        assert_eq!(outcome, SyncOutcome::Replaced);
        assert!(!std::fs::symlink_metadata(&dest).unwrap().is_symlink());
        assert_eq!(std::fs::read_to_string(&source).unwrap(), "generated");
    }

    #[test]
    fn refuses_to_replace_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".zshrc");
        std::fs::create_dir(&dest).unwrap();

        let err = write_file(ConflictPolicy::Overwrite, &dest, "generated").unwrap_err();

        assert!(err.to_string().contains("refusing to replace directory"));
    }
}
