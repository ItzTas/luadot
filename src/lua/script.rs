use std::path::Path;
use std::sync::MutexGuard;

use anyhow::{Context, Result, anyhow};

use super::constants::MODULES_DIR;
use super::ld::{API, Paths, Surface, install, share};
use super::runtime::{add_module_path, runtime};
use super::{Config, Shared};
use crate::state::Classes;

pub fn run_script(
    command: &str,
    surface: Surface,
    path: &Path,
    modules: &[&Path],
    paths: &Paths,
    classes: &Classes,
    config: &Shared,
) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("{command}: failed to read {}", path.display()))?;

    run_source(
        command,
        surface,
        &source,
        &path.display().to_string(),
        modules,
        paths,
        classes,
        config,
    )
}

pub fn run_source(
    command: &str,
    surface: Surface,
    source: &str,
    name: &str,
    modules: &[&Path],
    paths: &Paths,
    classes: &Classes,
    config: &Shared,
) -> Result<()> {
    let lua = runtime().with_context(|| format!("{command}: failed to start the Lua runtime"))?;
    install(&lua, surface, paths, classes)
        .with_context(|| format!("{command}: failed to install `{API}`"))?;
    share(&lua, config);

    for dir in modules.iter().rev() {
        add_module_path(&lua, dir)
            .with_context(|| format!("{command}: failed to make {MODULES_DIR}/ requirable"))?;
    }

    lua.load(source)
        .set_name(name)
        .exec()
        .with_context(|| format!("{command}: failed to run {name}"))?;

    lua.remove_app_data::<Shared>();
    kept(command, config)?.keep_runtime(lua);

    Ok(())
}

fn kept<'a>(command: &str, config: &'a Shared) -> Result<MutexGuard<'a, Config>> {
    config
        .lock()
        .map_err(|_| anyhow!("{command}: the configuration is still being changed"))
}
