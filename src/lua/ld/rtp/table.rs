use mlua::{Lua, Table};

use super::super::table::{Builder, build};
use super::add;
use super::constants::ADD;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 1] = [(ADD, add::function)];

    build(lua, &functions)
}
