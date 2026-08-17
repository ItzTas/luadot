use mlua::{Function, Lua};

use super::super::constants::API;
use super::super::path::Paths;
use super::constants::{LIST, NAMESPACE};
use super::scripts::listing;

pub fn function(lua: &Lua, paths: &Paths) -> mlua::Result<Function> {
    let paths = paths.clone();
    let command = format!("`{API}.{NAMESPACE}.{LIST}`");

    lua.create_function(move |_, ()| listing(&paths, &command).map(|(_, names)| names))
}
