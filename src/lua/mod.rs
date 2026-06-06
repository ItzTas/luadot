#![allow(dead_code)]

use mlua::Lua;

pub fn runtime() -> mlua::Result<Lua> {
    Ok(Lua::new())
}
