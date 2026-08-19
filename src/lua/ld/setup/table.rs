use mlua::{Function, Lua, Table};

use super::super::class;
use super::super::constants::API;
use super::super::path::Paths;
use super::super::repo::require;
use super::super::surface;
use super::constants::{ALL, LIST, NAMESPACE};
use super::{all, list, scripts};

pub fn table(lua: &Lua, paths: &Paths) -> mlua::Result<Table> {
    let setup = lua.create_table()?;
    setup.set(LIST, list::function(lua, paths)?)?;
    setup.set(ALL, all::function(lua, paths)?)?;

    let meta = lua.create_table()?;
    meta.set("__call", run(lua, paths)?)?;
    setup.set_metatable(Some(meta))?;

    Ok(setup)
}

fn run(lua: &Lua, paths: &Paths) -> mlua::Result<Function> {
    let paths = paths.clone();
    let command = format!("`{API}.{NAMESPACE}`");

    lua.create_function(move |lua, (_, name): (Table, String)| {
        surface::slow(lua, NAMESPACE);

        let repo = require(paths.repo(), &command)?;
        let classes = class::current(lua);

        scripts::run(&paths, &command, repo, &name, &classes)
    })
}
