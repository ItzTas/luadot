use std::path::Path;

use anyhow::{Context, Result};

use super::constants::MODULES_DIR;
use super::ld::{API, Paths, Surface, install};
use super::runtime::{add_module_path, runtime};
use crate::state::Classes;

pub fn run_script(
    command: &str,
    surface: Surface,
    path: &Path,
    modules: &[&Path],
    paths: &Paths,
    classes: &Classes,
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
) -> Result<()> {
    let lua = runtime().with_context(|| format!("{command}: failed to start the Lua runtime"))?;
    install(&lua, surface, paths, classes)
        .with_context(|| format!("{command}: failed to install `{API}`"))?;

    for dir in modules.iter().rev() {
        add_module_path(&lua, dir)
            .with_context(|| format!("{command}: failed to make {MODULES_DIR}/ requirable"))?;
    }

    lua.load(source)
        .set_name(name)
        .exec()
        .with_context(|| format!("{command}: failed to run {name}"))
}
