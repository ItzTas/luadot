use mlua::{Lua, Table};

use super::super::path::Paths;
use super::constants::{AT, CLONE};
use super::{at, clone, run};

pub fn table(lua: &Lua, paths: &Paths) -> mlua::Result<Table> {
    let git = lua.create_table()?;
    git.set(CLONE, clone::function(lua)?)?;
    git.set(AT, at::function(lua)?)?;

    let meta = lua.create_table()?;
    meta.set("__call", run::function(lua, paths)?)?;
    git.set_metatable(Some(meta))?;

    Ok(git)
}
