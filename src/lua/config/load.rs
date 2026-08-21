use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result, anyhow};
use tracing::debug;

use super::constants::CONFIG_FILE;
use super::types::{Config, Shared};
use crate::lua::constants::MODULES_DIR;
use crate::lua::ld::{API, Paths, Surface, install};
use crate::lua::runtime::{add_module_path, runtime};
use crate::state::{self, Classes};
use crate::utils;

pub fn load_config() -> Result<Shared> {
    load_from(&config_path()?)
}

#[cfg(test)]
pub fn from_source(source: &str) -> Result<Config> {
    from_classes(source, &Classes::default())
}

#[cfg(test)]
pub fn from_classes(source: &str, classes: &Classes) -> Result<Config> {
    let shared = run(source, "test", None, None, classes)?;
    let config = shared
        .lock()
        .map_err(|_| anyhow!("config: the configuration is still being changed"))?
        .clone();

    Ok(config)
}

pub fn config_path() -> Result<PathBuf> {
    Ok(utils::config_dir()?.join(CONFIG_FILE))
}

fn load_from(path: &Path) -> Result<Shared> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            debug!(path = %path.display(), "no configuration file, using the defaults");
            return Ok(Arc::new(Mutex::new(Config::default())));
        }
        Err(err) => {
            return Err(err).with_context(|| format!("config: failed to read {}", path.display()));
        }
    };
    debug!(path = %path.display(), "running the configuration");
    let state = state::load()?;
    run(
        &source,
        &path.display().to_string(),
        path.parent(),
        state.repo(),
        state.classes(),
    )
}

fn run(
    source: &str,
    name: &str,
    dir: Option<&Path>,
    repo: Option<&Path>,
    classes: &Classes,
) -> Result<Shared> {
    let home = utils::home_dir().context("config: failed to locate your home directory")?;
    let dirs = utils::config_dir().context("config: failed to locate the configuration")?;

    let lua = runtime().context("config: failed to start the Lua runtime")?;
    let paths = Paths::new(&home, &dirs).with_repo(repo);
    install(&lua, Surface::Config, &paths, classes)
        .with_context(|| format!("config: failed to install `{API}`"))?;

    if let Some(dir) = dir {
        add_module_path(&lua, dir)
            .with_context(|| format!("config: failed to make {MODULES_DIR}/ requirable"))?;
    }

    lua.load(source)
        .set_name(name)
        .exec()
        .with_context(|| format!("config: failed to run {name}"))?;

    let shared = lua
        .remove_app_data::<Shared>()
        .context("config: the configuration was lost while running the script")?;
    shared
        .lock()
        .map_err(|_| anyhow!("config: the configuration is still being changed"))?
        .keep_runtime(lua);

    Ok(shared)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::LinkMode;

    fn loaded(shared: Shared) -> Config {
        shared.lock().unwrap().clone()
    }

    #[test]
    fn missing_file_yields_the_default_config() {
        let dir = tempfile::tempdir().unwrap();

        let config = loaded(load_from(&dir.path().join(CONFIG_FILE)).unwrap());

        assert_eq!(config.link_mode(Path::new(".bashrc")), LinkMode::Hard);
    }

    #[test]
    fn loads_the_config_file_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CONFIG_FILE);
        std::fs::write(&path, r#"ld.opt.link("symbolic")"#).unwrap();

        let config = loaded(load_from(&path).unwrap());

        assert_eq!(config.link_mode(Path::new(".bashrc")), LinkMode::Symbolic);
    }

    #[test]
    fn requires_modules_from_the_lua_directory() {
        let dir = tempfile::tempdir().unwrap();
        let modules = dir.path().join(MODULES_DIR);
        std::fs::create_dir_all(modules.join("editors")).unwrap();
        std::fs::write(
            modules.join("patterns.lua"),
            r#"return { { match = "*.swp", ignore = true }, { match = ".cache/**", ignore = true } }"#,
        )
        .unwrap();
        std::fs::write(
            modules.join("editors/init.lua"),
            r#"ld.rules({ { match = ".config/nvim/**", link = "symbolic" } })"#,
        )
        .unwrap();
        let path = dir.path().join(CONFIG_FILE);
        std::fs::write(
            &path,
            r#"
            ld.rules(require("patterns"))
            require("editors")
            "#,
        )
        .unwrap();

        let config = loaded(load_from(&path).unwrap());

        assert!(config.is_ignored(Path::new(".vimrc.swp")));
        assert!(config.is_ignored(Path::new(".cache/nvim/log")));
        assert_eq!(
            config.link_mode(Path::new(".config/nvim/init.lua")),
            LinkMode::Symbolic
        );
    }

    #[test]
    fn a_required_module_can_return_a_configuring_function() {
        let dir = tempfile::tempdir().unwrap();
        let modules = dir.path().join(MODULES_DIR);
        std::fs::create_dir_all(&modules).unwrap();
        std::fs::write(
            modules.join("dots.lua"),
            r#"
            local dots = {}

            function dots.setup(mode)
              ld.opt.link(mode)
            end

            return dots
            "#,
        )
        .unwrap();
        let path = dir.path().join(CONFIG_FILE);
        std::fs::write(&path, r#"require("dots").setup("symbolic")"#).unwrap();

        let config = loaded(load_from(&path).unwrap());

        assert_eq!(config.link_mode(Path::new(".bashrc")), LinkMode::Symbolic);
    }

    #[test]
    fn reports_a_module_that_cannot_be_required() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CONFIG_FILE);
        std::fs::write(&path, r#"require("missing")"#).unwrap();

        let err = format!("{:#}", load_from(&path).unwrap_err());

        assert!(err.contains("module 'missing' not found"));
    }

    #[test]
    fn rejects_a_broken_script() {
        let err = format!("{:#}", from_source("ld.opt.link(").unwrap_err());

        assert!(err.contains("failed to run"));
    }
}
