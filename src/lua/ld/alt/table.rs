use mlua::{Lua, Table};

use super::super::table::{Builder, build};
use super::constants::{CONCAT, EXISTS, EXPAND, FILE, GLOB, JSON, OUT, READ, RENDER};
use super::{concat, exists, expand, file, glob, json, out, read, render};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 9] = [
        (OUT, out::function),
        (FILE, file::function),
        (RENDER, render::function),
        (EXPAND, expand::function),
        (READ, read::function),
        (EXISTS, exists::function),
        (GLOB, glob::function),
        (CONCAT, concat::function),
        (JSON, json::function),
    ];

    build(lua, &functions)
}
