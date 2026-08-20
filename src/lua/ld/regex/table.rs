use mlua::{Lua, Table};

use super::super::table::{Builder, build};
use super::constants::{ESCAPE, FIND, GMATCH, GSUB, MATCH, SPLIT, TEST};
use super::{escape, find, gmatch, gsub, r#match, split, test};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 7] = [
        (TEST, test::function),
        (MATCH, r#match::function),
        (FIND, find::function),
        (GMATCH, gmatch::function),
        (GSUB, gsub::function),
        (SPLIT, split::function),
        (ESCAPE, escape::function),
    ];

    build(lua, &functions)
}
