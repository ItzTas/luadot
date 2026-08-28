use mlua::{Lua, LuaSerdeExt, Table};

use super::super::table::{Builder, build};
use super::constants::{DECODE, ENCODE, NULL};
use super::{decode, encode};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 2] = [(ENCODE, encode::function), (DECODE, decode::function)];

    let table = build(lua, &functions)?;
    table.set(NULL, lua.null())?;

    Ok(table)
}
