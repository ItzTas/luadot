use mlua::{Lua, Table};

use super::super::table::{Builder, build};
use super::constants::{EXISTS, EXPAND, FILE, GLOB, JSON, OUT, READ, RENDER};
use super::{exists, expand, file, glob, json, out, read, render};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 8] = [
        (OUT, out::function),
        (FILE, file::function),
        (RENDER, render::function),
        (EXPAND, expand::function),
        (READ, read::function),
        (EXISTS, exists::function),
        (GLOB, glob::function),
        (JSON, json::function),
    ];

    build(lua, &functions)
}
