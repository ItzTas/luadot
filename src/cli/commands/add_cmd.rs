use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::files::{self, LinkMode};
use crate::state;

pub fn add_cmd(args: &[String]) -> Result<()> {
    let source = args.first().context("add: missing file path")?;
    let source = Path::new(source);

    let repo = repo_dir()?;
    if !repo.is_dir() {
        bail!(
            "add: repository {} does not exist; run `luadot clone <url>` first",
            repo.display()
        );
    }

    let dest = destination(&repo, source)?;

    // Hard links are the only strategy for now; a future Lua configuration will
    // choose between hard and symbolic links per file.
    add(LinkMode::Hard, source, &dest)
}

fn add(mode: LinkMode, source: &Path, dest: &Path) -> Result<()> {
    if !source.is_file() {
        bail!("add: {} is not a file", source.display());
    }
    if dest.exists() {
        bail!("add: {} already exists in the repository", dest.display());
    }
    files::link(mode, source, dest)
}

fn destination(repo: &Path, source: &Path) -> Result<PathBuf> {
    let name = source
        .file_name()
        .with_context(|| format!("add: cannot determine file name of {}", source.display()))?;
    Ok(repo.join(name))
}

fn repo_dir() -> Result<PathBuf> {
    let state = state::load()?;
    state
        .repo
        .context("add: no repository set; run `luadot clone <url>` first")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_joins_repo_and_file_name() {
        let dest = destination(Path::new("/repo"), Path::new("/home/u/.bashrc")).unwrap();
        assert_eq!(dest, PathBuf::from("/repo/.bashrc"));
    }

    #[test]
    fn destination_errors_without_a_file_name() {
        assert!(destination(Path::new("/repo"), Path::new("/")).is_err());
    }

    #[test]
    fn add_hard_links_source_into_dest() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&source, "hello").unwrap();

        add(LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "hello");
    }

    #[test]
    fn add_errors_when_source_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("missing.txt");
        let dest = dir.path().join("dest.txt");

        let err = add(LinkMode::Hard, &source, &dest).unwrap_err();
        assert!(err.to_string().contains("is not a file"));
    }

    #[test]
    fn add_errors_when_dest_already_exists() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&source, "hello").unwrap();
        std::fs::write(&dest, "existing").unwrap();

        let err = add(LinkMode::Hard, &source, &dest).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }
}
