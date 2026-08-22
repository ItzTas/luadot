use mlua::{Lua, Table};

use super::super::table::{Builder, build};
use super::constants::PAGE;
use super::page;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 1] = [(PAGE, page::function)];

    build(lua, &functions)
}
