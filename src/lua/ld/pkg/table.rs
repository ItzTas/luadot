use mlua::{Lua, Table};

use super::super::table::{Builder, build};
use super::constants::INSTALL;
use super::install;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 1] = [(INSTALL, install::function)];

    build(lua, &functions)
}
