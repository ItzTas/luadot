use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::files::{self, LinkMode};
use crate::{state, utils};

pub fn add_cmd(args: &[String]) -> Result<()> {
    if args.is_empty() {
        bail!("add: missing path");
    }

    let repo = repo_dir()?;
    if !repo.is_dir() {
        bail!(
            "add: repository {} does not exist; run `luadot clone <url>` first",
            repo.display()
        );
    }

    let home = utils::home_dir()?;

    // Hard links are the only strategy for now; a future Lua configuration will
    // choose between hard and symbolic links per file.
    for (source, dest) in plan(&home, &repo, args)? {
        link_into_repo(LinkMode::Hard, &source, &dest)?;
    }
    Ok(())
}

/// Builds the `(source file, repository destination)` pairs to link.
///
/// Each destination mirrors the file's location under `home`. A directory is
/// walked recursively and every file it contains is mapped the same way. Fails
/// when a source is neither a file nor a directory, when a destination already
/// exists in the repository, or when two sources would map to the same
/// destination.
fn plan(home: &Path, repo: &Path, sources: &[String]) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
    for source in sources {
        let source =
            std::path::absolute(source).with_context(|| format!("add: invalid path {source}"))?;
        if source.is_dir() {
            collect_dir(home, repo, &source, &mut pairs)?;
        } else if source.is_file() {
            let dest = utils::repo_path(home, repo, &source)?;
            pairs.push((source, dest));
        } else {
            bail!("add: {} is not a file or directory", source.display());
        }
    }
    check_conflicts(&pairs)?;
    Ok(pairs)
}

/// Appends every file under `dir` to `pairs`, mirroring its home layout.
fn collect_dir(
    home: &Path,
    repo: &Path,
    dir: &Path,
    pairs: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<()> {
    // Reject a directory outside home before walking it; this also reports an
    // out-of-home empty directory that would otherwise be silently ignored.
    utils::repo_path(home, repo, dir)?;

    let mut files = Vec::new();
    walk(dir, &mut files)?;
    files.sort();
    for file in files {
        let dest = utils::repo_path(home, repo, &file)?;
        pairs.push((file, dest));
    }
    Ok(())
}

/// Collects the regular files under `dir` recursively. Symbolic links and other
/// non-regular entries are skipped, which also keeps the walk free of cycles.
fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("add: failed to read {}", dir.display()))?;
    for entry in entries {
        let entry = entry.with_context(|| format!("add: failed to read {}", dir.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("add: failed to inspect {}", entry.path().display()))?;
        let path = entry.path();
        if file_type.is_dir() {
            walk(&path, out)?;
        } else if file_type.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

/// Ensures no destination already exists in the repository and that no two
/// sources map to the same destination.
fn check_conflicts(pairs: &[(PathBuf, PathBuf)]) -> Result<()> {
    let mut seen: HashSet<&Path> = HashSet::new();
    for (_, dest) in pairs {
        if dest.exists() {
            bail!("add: {} already exists in the repository", dest.display());
        }
        if !seen.insert(dest.as_path()) {
            bail!("add: {} would be added more than once", dest.display());
        }
    }
    Ok(())
}

/// Hard links `source` into `dest`, creating the parent directories first.
fn link_into_repo(mode: LinkMode, source: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("add: failed to create {}", parent.display()))?;
    }
    files::link(mode, source, dest)
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

    fn arg(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn plan_maps_a_file_mirroring_home() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let source = home.join(".bashrc");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(&source, "x").unwrap();

        let pairs = plan(&home, &repo, &[arg(&source)]).unwrap();

        assert_eq!(pairs, vec![(source, repo.join(".bashrc"))]);
    }

    #[test]
    fn plan_maps_multiple_files_in_argument_order() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        let a = home.join(".a");
        let b = home.join(".b");
        std::fs::write(&a, "a").unwrap();
        std::fs::write(&b, "b").unwrap();

        let pairs = plan(&home, &repo, &[arg(&a), arg(&b)]).unwrap();

        assert_eq!(pairs, vec![(a, repo.join(".a")), (b, repo.join(".b"))]);
    }

    #[test]
    fn plan_walks_a_directory_mirroring_home() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = home.join(".config").join("nvim");
        std::fs::create_dir_all(nvim.join("lua")).unwrap();
        let init = nvim.join("init.lua");
        let plugins = nvim.join("lua").join("plugins.lua");
        std::fs::write(&init, "init").unwrap();
        std::fs::write(&plugins, "plugins").unwrap();

        let pairs = plan(&home, &repo, &[arg(&nvim)]).unwrap();

        assert_eq!(
            pairs,
            vec![
                (init, repo.join(".config").join("nvim").join("init.lua")),
                (
                    plugins,
                    repo.join(".config")
                        .join("nvim")
                        .join("lua")
                        .join("plugins.lua"),
                ),
            ]
        );
    }

    #[test]
    fn plan_skips_symlinks_inside_directories() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let cfg = home.join(".config");
        std::fs::create_dir_all(&cfg).unwrap();
        let real = cfg.join("real");
        std::fs::write(&real, "r").unwrap();
        std::os::unix::fs::symlink(&real, cfg.join("link")).unwrap();

        let pairs = plan(&home, &repo, &[arg(&cfg)]).unwrap();

        assert_eq!(pairs, vec![(real, repo.join(".config").join("real"))]);
    }

    #[test]
    fn plan_errors_when_source_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let missing = home.join("missing");

        let err = plan(&home, &repo, &[arg(&missing)])
            .unwrap_err()
            .to_string();
        assert!(err.contains("is not a file or directory"));
    }

    #[test]
    fn plan_errors_when_destination_exists() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::write(repo.join(".bashrc"), "old").unwrap();
        let source = home.join(".bashrc");
        std::fs::write(&source, "new").unwrap();

        let err = plan(&home, &repo, &[arg(&source)])
            .unwrap_err()
            .to_string();
        assert!(err.contains("already exists"));
    }

    #[test]
    fn plan_errors_on_duplicate_destinations() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        let source = home.join(".bashrc");
        std::fs::write(&source, "x").unwrap();

        let err = plan(&home, &repo, &[arg(&source), arg(&source)])
            .unwrap_err()
            .to_string();
        assert!(err.contains("more than once"));
    }

    #[test]
    fn link_into_repo_hard_links_source_into_dest() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&source, "hello").unwrap();

        link_into_repo(LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "hello");
    }

    #[test]
    fn link_into_repo_creates_missing_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("nested").join("deep").join("dest.txt");
        std::fs::write(&source, "hello").unwrap();

        link_into_repo(LinkMode::Hard, &source, &dest).unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "hello");
    }
}
