use mlua::{Lua, Table};

use super::super::table::{Builder, build};
use super::constants::{FILE, OUT, RENDER};
use super::{file, out, render};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 3] = [
        (OUT, out::function),
        (FILE, file::function),
        (RENDER, render::function),
    ];

    build(lua, &functions)
}
