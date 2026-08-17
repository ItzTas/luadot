use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::crypt;
use crate::files;
use crate::git;
use crate::lua::{self, Config};
use crate::utils;

#[derive(Debug, Args)]
pub struct AddArgs {
    #[arg(value_name = "PATH", required = true)]
    pub paths: Vec<String>,
}

pub fn add_cmd(args: AddArgs) -> Result<()> {
    let config = lua::load_config()?;
    let repo = utils::require_repo("add", config.repo_dir())?;

    let home = utils::home_dir()?;

    for (source, dest) in plan(&home, &repo, &args.paths, &config)? {
        link_into_repo(&config, &repo, &source, &dest)?;
    }
    Ok(())
}

fn plan(
    home: &Path,
    repo: &Path,
    sources: &[String],
    config: &Config,
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut excludes = git::Excludes::open("add", repo)?;

    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
    for source in sources {
        let source =
            std::path::absolute(source).with_context(|| format!("add: invalid path {source}"))?;
        pairs.extend(collect(home, repo, source, config, &mut excludes)?);
    }
    check_conflicts(&pairs)?;
    Ok(pairs)
}

fn collect(
    home: &Path,
    repo: &Path,
    source: PathBuf,
    config: &Config,
    excludes: &mut git::Excludes,
) -> Result<Vec<(PathBuf, PathBuf)>> {
    if source.is_dir() {
        check_gitignore(home, &source, git::Kind::Directory, excludes)?;
        return collect_dir(home, repo, &source, config, excludes);
    }
    if source.is_file() {
        check_gitignore(home, &source, git::Kind::File, excludes)?;
        let Some(pair) = pair(home, repo, source, config, excludes)? else {
            return Ok(Vec::new());
        };
        return Ok(vec![pair]);
    }
    bail!("add: {} is not a file or directory", source.display())
}

fn collect_dir(
    home: &Path,
    repo: &Path,
    dir: &Path,
    config: &Config,
    excludes: &mut git::Excludes,
) -> Result<Vec<(PathBuf, PathBuf)>> {
    utils::repo_path(home, repo, dir)?;

    let mut files = Vec::new();
    walk(dir, &mut files)?;
    files.sort();

    let mut pairs = Vec::new();
    for file in files {
        if let Some(pair) = pair(home, repo, file, config, excludes)? {
            pairs.push(pair);
        }
    }
    Ok(pairs)
}

fn check_gitignore(
    home: &Path,
    source: &Path,
    kind: git::Kind,
    excludes: &mut git::Excludes,
) -> Result<()> {
    let relative = utils::managed_relative(home, source)?;
    if !excludes.excluded(&relative, kind)? {
        return Ok(());
    }

    bail!(
        "add: {} lands on {}, which the repository's .gitignore excludes",
        source.display(),
        relative.display()
    )
}

fn pair(
    home: &Path,
    repo: &Path,
    source: PathBuf,
    config: &Config,
    excludes: &mut git::Excludes,
) -> Result<Option<(PathBuf, PathBuf)>> {
    let dest = utils::repo_path(home, repo, &source)?;

    let relative = utils::relative(repo, &dest);
    if config.is_ignored(relative) || excludes.excluded(relative, git::Kind::File)? {
        return Ok(None);
    }
    if !config.encrypt(relative) {
        return Ok(Some((source, dest)));
    }
    if utils::is_root(relative) {
        bail!(
            "add: {} matches an encrypt rule, and encrypted system files are not supported",
            source.display()
        );
    }

    let stored = crypt::stored(&dest, config.crypt_backend());
    Ok(Some((source, stored)))
}

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
            continue;
        }
        if file_type.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

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

fn link_into_repo(config: &Config, repo: &Path, source: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("add: failed to create {}", parent.display()))?;
    }

    let relative = utils::relative(repo, dest);
    if let Some((stripped, backend)) = crypt::split(relative)
        && config.encrypt(&stripped)
    {
        return crypt::encrypt("add", backend, config.crypt_recipients(), source, dest);
    }
    if utils::is_root(relative) {
        return files::import_system(source, dest);
    }
    files::link(config.link_mode(relative), source, dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arg(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    fn gitignore(repo: &Path, rules: &str) {
        gix::init(repo).unwrap();
        std::fs::write(repo.join(".gitignore"), rules).unwrap();
    }

    #[test]
    fn plan_drops_the_files_the_configuration_ignores() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        let kept = home.join(".vimrc");
        let ignored = home.join(".vimrc.swp");
        std::fs::write(&kept, "a").unwrap();
        std::fs::write(&ignored, "b").unwrap();

        let config =
            lua::from_source(r#"ld.rules({ match = "home/*.swp", ignore = true })"#).unwrap();
        let pairs = plan(&home, &repo, &[arg(&kept), arg(&ignored)], &config).unwrap();

        assert_eq!(pairs, vec![(kept, repo.join("home/.vimrc"))]);
    }

    #[test]
    fn plan_refuses_a_file_the_gitignore_excludes() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        gitignore(&repo, "home/*.swp\n");
        let source = home.join(".vimrc.swp");
        std::fs::write(&source, "b").unwrap();

        let err = plan(&home, &repo, &[arg(&source)], &Config::default())
            .unwrap_err()
            .to_string();

        assert!(err.contains("home/.vimrc.swp"));
        assert!(err.contains(".gitignore"));
    }

    #[test]
    fn plan_refuses_a_directory_the_gitignore_excludes() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let cache = home.join(".cache");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::write(cache.join("log"), "b").unwrap();
        gitignore(&repo, "home/.cache/\n");

        let err = plan(&home, &repo, &[arg(&cache)], &Config::default())
            .unwrap_err()
            .to_string();

        assert!(err.contains("home/.cache"));
        assert!(err.contains(".gitignore"));
    }

    #[test]
    fn plan_drops_the_walked_files_the_gitignore_excludes() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let nvim = home.join(".config").join("nvim");
        std::fs::create_dir_all(&nvim).unwrap();
        gitignore(&repo, "*.log\n");
        let init = nvim.join("init.lua");
        std::fs::write(&init, "init").unwrap();
        std::fs::write(nvim.join("lsp.log"), "noise").unwrap();

        let pairs = plan(&home, &repo, &[arg(&nvim)], &Config::default()).unwrap();

        assert_eq!(pairs, vec![(init, repo.join("home/.config/nvim/init.lua"))]);
    }

    #[test]
    fn plan_keeps_the_files_no_gitignore_pattern_reaches() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        gitignore(&repo, "home/*.swp\n");
        let source = home.join(".vimrc");
        std::fs::write(&source, "a").unwrap();

        let pairs = plan(&home, &repo, &[arg(&source)], &Config::default()).unwrap();

        assert_eq!(pairs, vec![(source, repo.join("home/.vimrc"))]);
    }

    #[test]
    fn plan_maps_a_file_mirroring_home() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let source = home.join(".bashrc");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(&source, "x").unwrap();

        let pairs = plan(&home, &repo, &[arg(&source)], &Config::default()).unwrap();

        assert_eq!(pairs, vec![(source, repo.join("home/.bashrc"))]);
    }

    #[test]
    fn plan_maps_a_system_file_under_the_root_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let source = dir.path().join("etc/pacman.conf");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "x").unwrap();

        let pairs = plan(&home, &repo, &[arg(&source)], &Config::default()).unwrap();

        let relative = source.strip_prefix("/").unwrap();
        assert_eq!(
            pairs,
            vec![(source.clone(), repo.join("root").join(relative))]
        );
    }

    #[test]
    fn plan_stores_an_encrypted_file_under_the_backend_extension() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&home).unwrap();
        let source = home.join(".netrc");
        std::fs::write(&source, "machine example").unwrap();

        let config = lua::from_source(
            r#"
            ld.crypt.backend("gpg")
            ld.rules({ match = "home/.netrc", encrypt = true })
            "#,
        )
        .unwrap();
        let pairs = plan(&home, &repo, &[arg(&source)], &config).unwrap();

        assert_eq!(pairs, vec![(source, repo.join("home/.netrc.gpg"))]);
    }

    #[test]
    fn plan_refuses_an_encrypted_system_file() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let source = dir.path().join("etc/secret.conf");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "secret").unwrap();

        let config =
            lua::from_source(r#"ld.rules({ match = "root/**", encrypt = true })"#).unwrap();
        let err = plan(&home, &repo, &[arg(&source)], &config)
            .unwrap_err()
            .to_string();

        assert!(err.contains("encrypted system files are not supported"));
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

        let pairs = plan(&home, &repo, &[arg(&a), arg(&b)], &Config::default()).unwrap();

        assert_eq!(
            pairs,
            vec![(a, repo.join("home/.a")), (b, repo.join("home/.b"))]
        );
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

        let pairs = plan(&home, &repo, &[arg(&nvim)], &Config::default()).unwrap();

        assert_eq!(
            pairs,
            vec![
                (init, repo.join("home/.config/nvim/init.lua")),
                (plugins, repo.join("home/.config/nvim/lua/plugins.lua")),
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

        let pairs = plan(&home, &repo, &[arg(&cfg)], &Config::default()).unwrap();

        assert_eq!(pairs, vec![(real, repo.join("home/.config/real"))]);
    }

    #[test]
    fn plan_errors_when_source_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let repo = dir.path().join("repo");
        let missing = home.join("missing");

        let err = plan(&home, &repo, &[arg(&missing)], &Config::default())
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
        std::fs::create_dir_all(repo.join("home")).unwrap();
        std::fs::write(repo.join("home/.bashrc"), "old").unwrap();
        let source = home.join(".bashrc");
        std::fs::write(&source, "new").unwrap();

        let err = plan(&home, &repo, &[arg(&source)], &Config::default())
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

        let err = plan(
            &home,
            &repo,
            &[arg(&source), arg(&source)],
            &Config::default(),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("more than once"));
    }

    #[test]
    fn link_into_repo_hard_links_source_into_dest() {
        use std::os::unix::fs::MetadataExt;

        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        let source = dir.path().join("source.txt");
        let dest = repo.join("home/dest.txt");
        std::fs::write(&source, "hello").unwrap();

        link_into_repo(&Config::default(), &repo, &source, &dest).unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "hello");
        assert_eq!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );
    }

    #[test]
    fn link_into_repo_copies_a_system_file() {
        use std::os::unix::fs::MetadataExt;

        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        let source = dir.path().join("pacman.conf");
        let dest = repo.join("root/etc/pacman.conf");
        std::fs::write(&source, "conf").unwrap();

        link_into_repo(&Config::default(), &repo, &source, &dest).unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "conf");
        assert_ne!(
            std::fs::metadata(&source).unwrap().ino(),
            std::fs::metadata(&dest).unwrap().ino()
        );
    }

    #[test]
    fn link_into_repo_creates_missing_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        let source = dir.path().join("source.txt");
        let dest = repo.join("home/nested/deep/dest.txt");
        std::fs::write(&source, "hello").unwrap();

        link_into_repo(&Config::default(), &repo, &source, &dest).unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "hello");
    }
}
