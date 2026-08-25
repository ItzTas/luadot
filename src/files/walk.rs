use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::constants::GIT_DIR;
use super::template::{is_template, template_target};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    File(PathBuf),
    Template(PathBuf),
    Standalone(PathBuf),
}

impl Entry {
    pub fn path(&self) -> &Path {
        match self {
            Self::File(path) | Self::Template(path) | Self::Standalone(path) => path,
        }
    }

    pub fn target(&self) -> PathBuf {
        match self {
            Self::File(path) => path.clone(),
            Self::Template(path) | Self::Standalone(path) => {
                template_target(path).unwrap_or_else(|| path.clone())
            }
        }
    }
}

pub fn collect_entries(command: &str, root: &Path) -> Result<Vec<Entry>> {
    if !root.is_dir() {
        if is_template(root) {
            return Ok(vec![Entry::Standalone(root.to_path_buf())]);
        }
        return Ok(vec![Entry::File(root.to_path_buf())]);
    }

    if is_template(root) {
        return Ok(vec![Entry::Template(root.to_path_buf())]);
    }

    let mut entries = Vec::new();
    collect_into(command, root, &mut entries)?;
    entries.sort_by(|left, right| left.path().cmp(right.path()));
    Ok(entries)
}

pub fn collect_files(command: &str, root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in collect_entries(command, root)? {
        match entry {
            Entry::File(path) | Entry::Standalone(path) => files.push(path),
            Entry::Template(dir) => files.extend(inside(command, &dir)?),
        }
    }
    files.sort();
    Ok(files)
}

fn inside(command: &str, dir: &Path) -> Result<Vec<PathBuf>> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("{command}: failed to read {}", dir.display()))?;

    let mut files = Vec::new();
    for entry in entries {
        let entry = entry
            .with_context(|| format!("{command}: failed to read an entry in {}", dir.display()))?;
        files.extend(collect_files(command, &entry.path())?);
    }
    Ok(files)
}

fn collect_into(command: &str, dir: &Path, entries: &mut Vec<Entry>) -> Result<()> {
    let read = std::fs::read_dir(dir)
        .with_context(|| format!("{command}: failed to read {}", dir.display()))?;
    for entry in read {
        let entry = entry
            .with_context(|| format!("{command}: failed to read an entry in {}", dir.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("{command}: failed to inspect {}", entry.path().display()))?;
        if !file_type.is_dir() {
            if is_template(&entry.path()) {
                entries.push(Entry::Standalone(entry.path()));
                continue;
            }
            entries.push(Entry::File(entry.path()));
            continue;
        }
        if entry.file_name() == GIT_DIR {
            continue;
        }
        if is_template(&entry.path()) {
            entries.push(Entry::Template(entry.path()));
            continue;
        }
        collect_into(command, &entry.path(), entries)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_files_skips_the_git_directory() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git/objects")).unwrap();
        std::fs::write(root.join(".git/config"), "x").unwrap();
        std::fs::write(root.join(".git/objects/blob"), "y").unwrap();
        std::fs::write(root.join(".vimrc"), "z").unwrap();

        assert_eq!(
            collect_files("apply", root).unwrap(),
            vec![root.join(".vimrc")]
        );
    }

    #[test]
    fn collect_entries_keeps_a_template_whole() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let template = root.join(".zshrc.luadot");
        std::fs::create_dir_all(template.join("nested")).unwrap();
        std::fs::write(template.join("luadot.lua"), "brain").unwrap();
        std::fs::write(template.join("nested/variant.zsh"), "variant").unwrap();
        std::fs::write(root.join(".vimrc"), "plain").unwrap();

        let entries = collect_entries("apply", root).unwrap();

        assert_eq!(
            entries,
            vec![Entry::File(root.join(".vimrc")), Entry::Template(template)]
        );
    }

    #[test]
    fn a_standalone_template_is_its_own_entry() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let standalone = root.join(".zprofile.luadot");
        std::fs::write(&standalone, "export HOST=<%= 1 %>\n").unwrap();
        std::fs::write(root.join(".vimrc"), "plain").unwrap();

        let entries = collect_entries("alt", root).unwrap();

        assert_eq!(
            entries,
            vec![
                Entry::File(root.join(".vimrc")),
                Entry::Standalone(standalone),
            ]
        );
    }
}
