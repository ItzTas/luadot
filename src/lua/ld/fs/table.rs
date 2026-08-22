use mlua::{Lua, Table};

use super::super::table::{Builder, build};
use super::constants::{EXISTS, IS_DIR, LS, MKDIR, READ, RM, WRITE};
use super::{exists, is_dir, ls, mkdir, read, rm, write};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 7] = [
        (EXISTS, exists::function),
        (IS_DIR, is_dir::function),
        (MKDIR, mkdir::function),
        (LS, ls::function),
        (RM, rm::function),
        (READ, read::function),
        (WRITE, write::function),
    ];

    build(lua, &functions)
}
