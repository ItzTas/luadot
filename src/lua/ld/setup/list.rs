use mlua::{Function, Lua};

use super::super::constants::API;
use super::super::parse::chain;
use super::super::path::Paths;
use super::super::repo::require;
use super::constants::{LIST, NAMESPACE};
use crate::lua::setup;

pub fn function(lua: &Lua, paths: &Paths) -> mlua::Result<Function> {
    let paths = paths.clone();
    let command = format!("`{API}.{NAMESPACE}.{LIST}`");

    lua.create_function(move |_, ()| {
        let repo = require(paths.repo(), &command)?;
        let dir = setup::setup_dir(&command, paths.home(), paths.config(), repo).map_err(chain)?;
        setup::list(&command, &dir).map_err(chain)
    })
}
