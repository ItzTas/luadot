use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::constants::COMMAND;
use super::fs::create_parent;
use super::link::{LinkMode, link};
use super::status::FileStatus;
use super::sync::{ConflictPolicy, SyncOutcome, refused, same_contents};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Shape {
    Directory,
    File,
    Link(PathBuf),
}

pub fn dir_status(mode: LinkMode, source: &Path, dest: &Path) -> Result<FileStatus> {
    refuse_hard(mode)?;

    let Ok(meta) = std::fs::symlink_metadata(dest) else {
        return Ok(FileStatus::Missing);
    };

    if mode == LinkMode::Symbolic && meta.file_type().is_symlink() {
        return Ok(match std::fs::read_link(dest)? == source {
            true => FileStatus::Synced,
            false => FileStatus::Differs,
        });
    }

    if !meta.is_dir() || !trees_equal(source, dest)? {
        return Ok(FileStatus::Differs);
    }

    Ok(match mode {
        LinkMode::Symbolic => FileStatus::Unlinked,
        _ => FileStatus::Synced,
    })
}

pub fn sync_dir(
    policy: ConflictPolicy,
    mode: LinkMode,
    source: &Path,
    dest: &Path,
) -> Result<SyncOutcome> {
    match dir_status(mode, source, dest)? {
        FileStatus::Synced => Ok(SyncOutcome::AlreadySynced),
        FileStatus::Missing => {
            place_dir(mode, source, dest)?;
            Ok(SyncOutcome::Created)
        }
        _ => {
            if let Some(outcome) = refused(COMMAND, policy, dest)? {
                return Ok(outcome);
            }
            remove_entry(COMMAND, dest)?;
            place_dir(mode, source, dest)?;
            Ok(SyncOutcome::Replaced)
        }
    }
}

pub fn copy_tree(command: &str, source: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)
        .with_context(|| format!("{command}: failed to create {}", dest.display()))?;

    let entries = std::fs::read_dir(source)
        .with_context(|| format!("{command}: failed to read {}", source.display()))?;
    for entry in entries {
        let entry =
            entry.with_context(|| format!("{command}: failed to read {}", source.display()))?;
        let from = entry.path();
        let into = dest.join(entry.file_name());
        let kind = entry
            .file_type()
            .with_context(|| format!("{command}: failed to inspect {}", from.display()))?;

        if kind.is_dir() {
            copy_tree(command, &from, &into)?;
            continue;
        }
        if kind.is_symlink() {
            let target = std::fs::read_link(&from)
                .with_context(|| format!("{command}: failed to read {}", from.display()))?;
            std::os::unix::fs::symlink(&target, &into).with_context(|| {
                format!(
                    "{command}: failed to link {} -> {}",
                    into.display(),
                    target.display()
                )
            })?;
            continue;
        }
        link(LinkMode::Copy, &from, &into)?;
    }

    Ok(())
}

pub fn remove_entry(command: &str, path: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)
        .with_context(|| format!("{command}: failed to inspect {}", path.display()))?;

    let removed = match meta.is_dir() {
        true => std::fs::remove_dir_all(path),
        false => std::fs::remove_file(path),
    };

    removed.with_context(|| format!("{command}: failed to remove {}", path.display()))
}

fn place_dir(mode: LinkMode, source: &Path, dest: &Path) -> Result<()> {
    create_parent(COMMAND, dest)?;

    match mode {
        LinkMode::Symbolic => link(LinkMode::Symbolic, source, dest),
        _ => copy_tree(COMMAND, source, dest),
    }
}

fn refuse_hard(mode: LinkMode) -> Result<()> {
    if mode != LinkMode::Hard {
        return Ok(());
    }

    bail!("{COMMAND}: a directory cannot be hard linked")
}

fn trees_equal(source: &Path, dest: &Path) -> Result<bool> {
    let ours = shapes(source)?;
    let theirs = shapes(dest)?;
    if ours != theirs {
        return Ok(false);
    }

    for (relative, shape) in &ours {
        if *shape != Shape::File {
            continue;
        }
        if !same_contents(&source.join(relative), &dest.join(relative))? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn shapes(root: &Path) -> Result<BTreeMap<PathBuf, Shape>> {
    let mut found = BTreeMap::new();
    fill(root, root, &mut found)?;

    Ok(found)
}

fn fill(root: &Path, dir: &Path, found: &mut BTreeMap<PathBuf, Shape>) -> Result<()> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("{COMMAND}: failed to read {}", dir.display()))?;
    for entry in entries {
        let entry =
            entry.with_context(|| format!("{COMMAND}: failed to read {}", dir.display()))?;
        let path = entry.path();
        let kind = entry
            .file_type()
            .with_context(|| format!("{COMMAND}: failed to inspect {}", path.display()))?;
        let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();

        if kind.is_dir() {
            found.insert(relative, Shape::Directory);
            fill(root, &path, found)?;
            continue;
        }
        if kind.is_symlink() {
            let target = std::fs::read_link(&path)
                .with_context(|| format!("{COMMAND}: failed to read {}", path.display()))?;
            found.insert(relative, Shape::Link(target));
            continue;
        }
        found.insert(relative, Shape::File);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, contents: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    fn tree(root: &Path) -> PathBuf {
        let source = root.join("repo/.config/nvim");
        write(&source.join("init.lua"), "init");
        write(&source.join("lua/plugins.lua"), "plugins");

        source
    }

    #[test]
    fn a_symbolic_directory_reports_its_states() {
        let dir = tempfile::tempdir().unwrap();
        let source = tree(dir.path());
        let dest = dir.path().join("home/.config/nvim");

        assert_eq!(
            dir_status(LinkMode::Symbolic, &source, &dest).unwrap(),
            FileStatus::Missing
        );

        std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&source, &dest).unwrap();
        assert_eq!(
            dir_status(LinkMode::Symbolic, &source, &dest).unwrap(),
            FileStatus::Synced
        );

        std::fs::remove_file(&dest).unwrap();
        copy_tree("files", &source, &dest).unwrap();
        assert_eq!(
            dir_status(LinkMode::Symbolic, &source, &dest).unwrap(),
            FileStatus::Unlinked
        );

        write(&dest.join("scratch.lua"), "scratch");
        assert_eq!(
            dir_status(LinkMode::Symbolic, &source, &dest).unwrap(),
            FileStatus::Differs
        );
    }

    #[test]
    fn a_copied_directory_is_synced_by_contents() {
        let dir = tempfile::tempdir().unwrap();
        let source = tree(dir.path());
        let dest = dir.path().join("home/.config/nvim");

        copy_tree("files", &source, &dest).unwrap();
        assert_eq!(
            dir_status(LinkMode::Copy, &source, &dest).unwrap(),
            FileStatus::Synced
        );

        write(&dest.join("init.lua"), "drifted");
        assert_eq!(
            dir_status(LinkMode::Copy, &source, &dest).unwrap(),
            FileStatus::Differs
        );
    }

    #[test]
    fn sync_dir_places_a_symlink_and_detects_it() {
        let dir = tempfile::tempdir().unwrap();
        let source = tree(dir.path());
        let dest = dir.path().join("home/.config/nvim");

        let outcome = sync_dir(
            ConflictPolicy::Overwrite,
            LinkMode::Symbolic,
            &source,
            &dest,
        )
        .unwrap();
        assert_eq!(outcome, SyncOutcome::Created);
        assert_eq!(std::fs::read_link(&dest).unwrap(), source);

        let outcome = sync_dir(
            ConflictPolicy::Overwrite,
            LinkMode::Symbolic,
            &source,
            &dest,
        )
        .unwrap();
        assert_eq!(outcome, SyncOutcome::AlreadySynced);
    }

    #[test]
    fn sync_dir_replaces_a_diverging_directory_under_the_policy() {
        let dir = tempfile::tempdir().unwrap();
        let source = tree(dir.path());
        let dest = dir.path().join("home/.config/nvim");
        write(&dest.join("init.lua"), "mine");

        let outcome = sync_dir(ConflictPolicy::Skip, LinkMode::Symbolic, &source, &dest).unwrap();
        assert_eq!(outcome, SyncOutcome::Skipped);
        assert_eq!(
            std::fs::read_to_string(dest.join("init.lua")).unwrap(),
            "mine"
        );

        let outcome = sync_dir(
            ConflictPolicy::Overwrite,
            LinkMode::Symbolic,
            &source,
            &dest,
        )
        .unwrap();
        assert_eq!(outcome, SyncOutcome::Replaced);
        assert_eq!(std::fs::read_link(&dest).unwrap(), source);
    }

    #[test]
    fn a_hard_linked_directory_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let source = tree(dir.path());
        let dest = dir.path().join("home/.config/nvim");

        let err = dir_status(LinkMode::Hard, &source, &dest)
            .unwrap_err()
            .to_string();

        assert_eq!(err, "files: a directory cannot be hard linked");
    }

    #[test]
    fn copy_tree_carries_a_symlink_as_it_is() {
        let dir = tempfile::tempdir().unwrap();
        let source = tree(dir.path());
        std::os::unix::fs::symlink("init.lua", source.join("entry.lua")).unwrap();
        let dest = dir.path().join("home/.config/nvim");

        copy_tree("files", &source, &dest).unwrap();

        assert_eq!(
            std::fs::read_link(dest.join("entry.lua")).unwrap(),
            PathBuf::from("init.lua")
        );
        assert_eq!(
            dir_status(LinkMode::Copy, &source, &dest).unwrap(),
            FileStatus::Synced
        );
    }
}
