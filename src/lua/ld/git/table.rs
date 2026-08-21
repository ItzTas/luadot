use mlua::{Lua, Table};

use super::super::path::Paths;
use super::run;

pub fn table(lua: &Lua, paths: &Paths) -> mlua::Result<Table> {
    let git = lua.create_table()?;

    let meta = lua.create_table()?;
    meta.set("__call", run::function(lua, paths)?)?;
    git.set_metatable(Some(meta))?;

    Ok(git)
}
