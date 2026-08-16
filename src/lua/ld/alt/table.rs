use mlua::{Lua, Table};

use super::super::table::{Builder, build};
use super::constants::{EXPAND, FILE, OUT, RENDER};
use super::{expand, file, out, render};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 4] = [
        (OUT, out::function),
        (FILE, file::function),
        (RENDER, render::function),
        (EXPAND, expand::function),
    ];

    build(lua, &functions)
}
