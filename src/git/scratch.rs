use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gix::ObjectId;
use gix::bstr::BStr;
use gix::index::{State, entry};
use gix::refs::transaction::{Change, PreviousValue, RefEdit};
use gix::refs::{Category, FullName, Target};

use super::constants::HEAD;

pub fn scratch(command: &str, dir: &Path, branch: &str) -> Result<()> {
    let repository = gix::init(dir).with_context(|| {
        format!(
            "{command}: failed to create a repository in {}",
            dir.display()
        )
    })?;

    let head = Category::LocalBranch
        .to_full_name(BStr::new(branch))
        .with_context(|| format!("{command}: {branch} is not a valid branch name"))?;
    let name: FullName = HEAD
        .try_into()
        .with_context(|| format!("{command}: {HEAD} is not a valid reference name"))?;

    repository
        .edit_reference(RefEdit {
            change: Change::Update {
                log: Default::default(),
                expected: PreviousValue::Any,
                new: Target::Symbolic(head),
            },
            name,
            deref: false,
        })
        .with_context(|| format!("{command}: failed to point {HEAD} at {branch}"))?;

    Ok(())
}

pub fn record(command: &str, dir: &Path, files: &[PathBuf]) -> Result<()> {
    let repository = gix::open(dir).with_context(|| {
        format!(
            "{command}: failed to open the repository in {}",
            dir.display()
        )
    })?;

    let mut state = State::new(repository.object_hash());
    for relative in files {
        let (stat, mode, id) = recorded(command, &repository, dir, relative)?;
        state.dangerously_push_entry(
            stat,
            id,
            entry::Flags::empty(),
            mode,
            gix::path::into_bstr(relative.as_path()).as_ref(),
        );
    }
    state.sort_entries();

    let mut index = gix::index::File::from_state(state, repository.index_path());
    index
        .write(gix::index::write::Options::default())
        .with_context(|| format!("{command}: failed to write the index of {}", dir.display()))?;

    Ok(())
}

fn recorded(
    command: &str,
    repository: &gix::Repository,
    dir: &Path,
    relative: &Path,
) -> Result<(entry::Stat, entry::Mode, ObjectId)> {
    let path = dir.join(relative);
    let failed = || format!("{command}: failed to read {}", path.display());

    let contents = std::fs::read(&path).with_context(failed)?;
    let id = repository
        .write_blob(contents)
        .with_context(|| format!("{command}: failed to record {}", path.display()))?;

    let metadata = gix::index::fs::Metadata::from_path_no_follow(&path).with_context(failed)?;
    let stat = entry::Stat::from_fs(&metadata).with_context(failed)?;
    let mode = match metadata.is_executable() {
        true => entry::Mode::FILE_EXECUTABLE,
        false => entry::Mode::FILE,
    };

    Ok((stat, mode, id.detach()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository(branch: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("tree");
        std::fs::create_dir(&root).unwrap();
        scratch("diff", &root, branch).unwrap();

        (dir, root)
    }

    fn write(root: &Path, relative: &str, contents: &str, mode: u32) {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;

        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(mode)
            .open(&path)
            .unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    #[test]
    fn a_scratch_repository_starts_on_the_branch_it_is_given() {
        let (_dir, root) = repository("luadot");
        let repository = gix::open(&root).unwrap();

        assert_eq!(repository.head_name().unwrap().unwrap().shorten(), "luadot");
        assert!(repository.head().unwrap().is_unborn());
    }

    #[test]
    fn recording_stages_every_file_it_is_given() {
        let (_dir, root) = repository("luadot");
        write(&root, ".bashrc", "managed\n", 0o644);
        write(&root, ".vimrc", "set number\n", 0o644);

        record(
            "diff",
            &root,
            &[PathBuf::from(".bashrc"), PathBuf::from(".vimrc")],
        )
        .unwrap();

        let repository = gix::open(&root).unwrap();
        let index = repository.index().unwrap();
        let staged: Vec<String> = index
            .entries()
            .iter()
            .map(|entry| entry.path(&index).to_string())
            .collect();

        assert_eq!(staged, [".bashrc", ".vimrc"]);
    }

    #[test]
    fn the_executable_bit_reaches_the_index() {
        let (_dir, root) = repository("luadot");
        write(&root, "plain", "text\n", 0o644);
        write(&root, "script", "#!/bin/sh\n", 0o755);

        record(
            "diff",
            &root,
            &[PathBuf::from("plain"), PathBuf::from("script")],
        )
        .unwrap();

        let repository = gix::open(&root).unwrap();
        let index = repository.index().unwrap();
        let modes: Vec<entry::Mode> = index.entries().iter().map(|entry| entry.mode).collect();

        assert_eq!(modes, [entry::Mode::FILE, entry::Mode::FILE_EXECUTABLE]);
    }

    #[test]
    fn the_staged_content_is_what_the_file_held() {
        let (_dir, root) = repository("luadot");
        write(&root, ".bashrc", "managed\n", 0o644);

        record("diff", &root, &[PathBuf::from(".bashrc")]).unwrap();

        let repository = gix::open(&root).unwrap();
        let index = repository.index().unwrap();
        let id = index.entries()[0].id;

        assert_eq!(repository.find_object(id).unwrap().data, b"managed\n");
    }
}
