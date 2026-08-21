use std::path::Path;

use super::super::parse::chain;
use super::super::path::Paths;
use super::super::repo::require;
use crate::lua::Shared;
use crate::lua::setup;
use crate::state::Classes;

pub fn listing<'a>(paths: &'a Paths, command: &str) -> mlua::Result<(&'a Path, Vec<String>)> {
    let repo = require(paths.repo(), command)?;
    let dir = setup::setup_dir(command, paths.home(), paths.config(), repo).map_err(chain)?;

    Ok((repo, setup::list(command, &dir).map_err(chain)?))
}

pub fn run(
    paths: &Paths,
    command: &str,
    repo: &Path,
    name: &str,
    classes: &Classes,
    shared: &Shared,
) -> mlua::Result<()> {
    setup::run_one(command, paths, repo, name, classes, shared).map_err(chain)
}
