use std::cell::RefCell;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::constants::{INIT_STEM, LUA_EXT, SETUP_DIR, SH_EXT};
use crate::lua::Shared;
use crate::lua::ld::{Paths, Surface};
use crate::lua::script::run_script;
use crate::state::{self, Classes};
use crate::utils;

thread_local! {
    static RUNNING: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

pub fn run_setups(command: &str, repo: &Path, names: &[String], shared: &Shared) -> Result<()> {
    let paths = Paths::new(
        &utils::home_dir()?,
        &utils::config_dir()?,
        &utils::data_dir()?,
    );
    let classes = state::load()?.classes().clone();

    for name in names {
        run_one(command, &paths, repo, name, &classes, shared)?;
    }
    Ok(())
}

pub fn list_setups(command: &str, repo: &Path) -> Result<Vec<String>> {
    let home = utils::home_dir()?;
    let config = utils::config_dir()?;
    list(command, &setup_dir(command, &home, &config, repo)?)
}

pub fn run_one(
    command: &str,
    paths: &Paths,
    repo: &Path,
    name: &str,
    classes: &Classes,
    shared: &Shared,
) -> Result<()> {
    let dir = setup_dir(command, paths.home(), paths.config(), repo)?;
    let Some(path) = find(&dir, name) else {
        bail!("{command}: no setup named `{name}` in {}", dir.display());
    };

    enter(command, name)?;
    let result = run_path(command, &dir, &path, paths, repo, classes, shared);
    leave(name);
    result
}

pub fn setup_dir(command: &str, home: &Path, config: &Path, repo: &Path) -> Result<PathBuf> {
    utils::repo_path(home, repo, &config.join(SETUP_DIR))
        .with_context(|| format!("{command}: failed to locate the setup directory"))
}

pub fn list(command: &str, dir: &Path) -> Result<Vec<String>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("{command}: failed to read {}", dir.display()));
        }
    };

    let mut names = BTreeSet::new();
    for entry in entries {
        let path = entry
            .with_context(|| format!("{command}: failed to read {}", dir.display()))?
            .path();
        if let Some(name) = setup_name(&path) {
            names.insert(name);
        }
    }
    Ok(names.into_iter().collect())
}

pub fn ordered(command: &str, names: Vec<String>, order: &[String]) -> Result<Vec<String>> {
    for name in order {
        if !names.contains(name) {
            bail!(
                "{command}: unknown setup `{name}` in order (available: {})",
                names.join(", ")
            );
        }
    }

    let rest = names.into_iter().filter(|name| !order.contains(name));
    Ok(order.iter().cloned().chain(rest).collect())
}

fn find(dir: &Path, name: &str) -> Option<PathBuf> {
    let entry = dir.join(name);
    let candidates = [
        dir.join(format!("{name}.{LUA_EXT}")),
        dir.join(format!("{name}.{SH_EXT}")),
        entry.join(format!("{INIT_STEM}.{LUA_EXT}")),
        entry.join(format!("{INIT_STEM}.{SH_EXT}")),
    ];

    candidates.into_iter().find(|candidate| candidate.is_file())
}

fn setup_name(path: &Path) -> Option<String> {
    if path.is_dir() {
        return match has_init(path) {
            true => file_part(path, Path::file_name),
            false => None,
        };
    }
    if !path.is_file() || !known_extension(path) {
        return None;
    }

    file_part(path, Path::file_stem)
}

fn file_part(path: &Path, part: fn(&Path) -> Option<&OsStr>) -> Option<String> {
    part(path).and_then(OsStr::to_str).map(str::to_string)
}

fn has_init(dir: &Path) -> bool {
    [LUA_EXT, SH_EXT]
        .into_iter()
        .any(|ext| dir.join(format!("{INIT_STEM}.{ext}")).is_file())
}

fn known_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|ext| [LUA_EXT, SH_EXT].contains(&ext))
}

fn run_path(
    command: &str,
    root: &Path,
    path: &Path,
    paths: &Paths,
    repo: &Path,
    classes: &Classes,
    shared: &Shared,
) -> Result<()> {
    if path.extension().is_none_or(|ext| ext != LUA_EXT) {
        return run_sh(command, path);
    }

    let modules = root
        .parent()
        .with_context(|| format!("{command}: {} has no parent directory", root.display()))?;
    let own = path.parent().filter(|parent| *parent != root);
    let mut paths = paths.clone().with_repo(Some(repo));
    if let Some(dir) = path.parent() {
        paths = paths.with_dir(dir);
    }
    let roots: Vec<&Path> = own.into_iter().chain([modules]).collect();

    run_script(
        command,
        Surface::Setup,
        path,
        &roots,
        &paths,
        classes,
        shared,
    )
}

fn run_sh(command: &str, path: &Path) -> Result<()> {
    let status = Command::new("sh")
        .arg(path)
        .status()
        .with_context(|| format!("{command}: failed to run {}", path.display()))?;

    if status.success() {
        return Ok(());
    }

    let code = status
        .code()
        .map_or_else(|| "signal".to_string(), |code| code.to_string());
    bail!("{command}: {} exited with status {code}", path.display());
}

fn enter(command: &str, name: &str) -> Result<()> {
    RUNNING.with(|running| {
        let mut running = running.borrow_mut();
        if running.iter().any(|current| current == name) {
            bail!(
                "{command}: setup `{name}` is already running (cycle: {} -> {name})",
                running.join(" -> ")
            );
        }
        running.push(name.to_string());
        Ok(())
    })
}

fn leave(name: &str) {
    RUNNING.with(|running| {
        let mut running = running.borrow_mut();
        if let Some(position) = running.iter().rposition(|current| current == name) {
            running.remove(position);
        }
    });
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::lua::{Config, Shared};

    fn configuration() -> Shared {
        Arc::new(Mutex::new(Config::default()))
    }

    use super::*;

    fn dirs(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let home = root.join("home");
        let config = home.join(".config/luadot");
        let repo = root.join("repo");
        (home, config, repo)
    }

    fn paths(home: &Path, config: &Path) -> Paths {
        Paths::new(home, config, &home.join(".local/share/luadot"))
    }

    fn write_setup(home: &Path, config: &Path, repo: &Path, file: &str, source: &str) -> PathBuf {
        let dir = setup_dir("test", home, config, repo).unwrap();
        let path = dir.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, source).unwrap();
        path
    }

    #[test]
    fn list_is_sorted_and_deduplicated() {
        let root = tempfile::tempdir().unwrap();
        let (home, config, repo) = dirs(root.path());
        write_setup(&home, &config, &repo, "ufw.lua", "");
        write_setup(&home, &config, &repo, "ufw.sh", "");
        write_setup(&home, &config, &repo, "docker.sh", "");
        write_setup(&home, &config, &repo, "notes.txt", "");

        let dir = setup_dir("test", &home, &config, &repo).unwrap();

        assert_eq!(list("test", &dir).unwrap(), ["docker", "ufw"]);
    }

    #[test]
    fn find_prefers_lua_over_sh() {
        let root = tempfile::tempdir().unwrap();
        let (home, config, repo) = dirs(root.path());
        write_setup(&home, &config, &repo, "ufw.sh", "");
        let lua = write_setup(&home, &config, &repo, "ufw.lua", "");
        let dir = setup_dir("test", &home, &config, &repo).unwrap();

        assert_eq!(find(&dir, "ufw").unwrap(), lua);
    }

    #[test]
    fn ordered_puts_the_request_first() {
        let names = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let order = ["c".to_string(), "a".to_string()];

        assert_eq!(ordered("test", names, &order).unwrap(), ["c", "a", "b"]);
    }

    #[test]
    fn runs_with_the_bootstrap_api() {
        let root = tempfile::tempdir().unwrap();
        let (home, config, repo) = dirs(root.path());
        write_setup(
            &home,
            &config,
            &repo,
            "ufw.lua",
            r#"
            local out = assert(io.open(ld.path.repo .. "/lua-ran.txt", "w"))
            out:write("done")
            out:close()
            "#,
        );

        run_one(
            "setup",
            &paths(&home, &config),
            &repo,
            "ufw",
            &Classes::default(),
            &configuration(),
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(repo.join("lua-ran.txt")).unwrap(),
            "done"
        );
    }

    #[test]
    fn runs_a_sh_setup() {
        let root = tempfile::tempdir().unwrap();
        let (home, config, repo) = dirs(root.path());
        let out = repo.join("sh-ran.txt");
        write_setup(
            &home,
            &config,
            &repo,
            "ufw.sh",
            &format!("printf done > {}", out.display()),
        );

        run_one(
            "setup",
            &paths(&home, &config),
            &repo,
            "ufw",
            &Classes::default(),
            &configuration(),
        )
        .unwrap();

        assert_eq!(std::fs::read_to_string(&out).unwrap(), "done");
    }

    #[test]
    fn a_cycle_is_reported() {
        let root = tempfile::tempdir().unwrap();
        let (home, config, repo) = dirs(root.path());
        write_setup(&home, &config, &repo, "loop.lua", r#"ld.setup("loop")"#);

        let err = format!(
            "{:#}",
            run_one(
                "setup",
                &paths(&home, &config),
                &repo,
                "loop",
                &Classes::default(),
                &configuration(),
            )
            .unwrap_err()
        );

        assert!(err.contains("already running"));
        assert!(err.contains("loop -> loop"));
    }
}
