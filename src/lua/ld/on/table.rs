use mlua::{Lua, Table};

use super::super::table::build;
use super::constants::FUNCTIONS;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    build(lua, &FUNCTIONS)
}
