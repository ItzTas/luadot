use mlua::{Lua, Table};

use super::super::table::build;
use super::constants::FUNCTIONS;
use super::line;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let print = build(lua, &FUNCTIONS)?;

    let meta = lua.create_table()?;
    meta.set("__call", line::function(lua)?)?;
    print.set_metatable(Some(meta))?;

    Ok(print)
}
