use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::constants::{CURRENT_DIR, LUA_EXT, SOURCE_NAME};
use crate::lua::Shared;
use crate::lua::ld::{Paths, Surface};
use crate::lua::script::{run_script, run_source};
use crate::state::{self, Classes};
use crate::utils;

enum Target {
    File(PathBuf),
    Source(String),
}

pub fn run_exec(command: &str, target: &str, shared: &Shared) -> Result<()> {
    let home =
        utils::home_dir().with_context(|| format!("{command}: failed to locate your home"))?;
    let config = utils::config_dir()
        .with_context(|| format!("{command}: failed to locate the configuration"))?;
    let data = utils::data_dir()
        .with_context(|| format!("{command}: failed to locate the data directory"))?;
    let state = state::load()?;
    let paths = Paths::new(&home, &config, &data).with_repo(state.repo());

    run(
        command,
        &classify(target, Path::new(target).is_file()),
        &paths,
        state.classes(),
        shared,
    )
}

fn classify(target: &str, is_file: bool) -> Target {
    let path = Path::new(target);
    let is_lua = path.extension().and_then(OsStr::to_str) == Some(LUA_EXT);

    if is_file || is_lua {
        return Target::File(path.to_path_buf());
    }

    Target::Source(target.to_string())
}

fn run(
    command: &str,
    target: &Target,
    paths: &Paths,
    classes: &Classes,
    shared: &Shared,
) -> Result<()> {
    match target {
        Target::File(path) => run_file(command, path, paths, classes, shared),
        Target::Source(source) => run_source(
            command,
            Surface::Exec,
            source,
            SOURCE_NAME,
            &[paths.config()],
            &paths.clone().with_dir(Path::new(CURRENT_DIR)),
            classes,
            shared,
        ),
    }
}

fn run_file(
    command: &str,
    path: &Path,
    paths: &Paths,
    classes: &Classes,
    shared: &Shared,
) -> Result<()> {
    let modules = modules(path);
    let paths = paths.clone().with_dir(&modules);

    run_script(
        command,
        Surface::Exec,
        path,
        &[&modules],
        &paths,
        classes,
        shared,
    )
}

fn modules(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from(CURRENT_DIR),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::lua::Config;

    use super::*;
    use crate::lua::constants::MODULES_DIR;

    fn paths(home: &Path) -> Paths {
        Paths::new(
            home,
            &home.join(".config/luadot"),
            &home.join(".local/share/luadot"),
        )
    }

    fn exec(target: &Target, home: &Path) -> Result<()> {
        run(
            "exec",
            target,
            &paths(home),
            &Classes::default(),
            &Arc::new(Mutex::new(Config::default())),
        )
    }

    #[test]
    fn a_source_string_requires_modules() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let modules = home.join(".config/luadot").join(MODULES_DIR);
        std::fs::create_dir_all(&modules).unwrap();
        std::fs::write(modules.join("greeting.lua"), r#"return "hello""#).unwrap();

        exec(
            &Target::Source(
                r#"
                local greeting = require("greeting")
                local out = assert(io.open(ld.path.home .. "/out.txt", "w"))
                out:write(greeting)
                out:close()
                "#
                .to_string(),
            ),
            &home,
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(home.join("out.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn a_file_requires_from_its_directory() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let script = dir.path().join("scripts/report.lua");
        let modules = script.parent().unwrap().join(MODULES_DIR);
        std::fs::create_dir_all(&modules).unwrap();
        std::fs::write(modules.join("greeting.lua"), r#"return "hello""#).unwrap();
        std::fs::write(
            &script,
            r#"
            local greeting = require("greeting")
            local out = assert(io.open(ld.path.home .. "/out.txt", "w"))
            out:write(greeting)
            out:close()
            "#,
        )
        .unwrap();

        exec(&Target::File(script), &home).unwrap();

        assert_eq!(
            std::fs::read_to_string(home.join("out.txt")).unwrap(),
            "hello"
        );
    }
}
