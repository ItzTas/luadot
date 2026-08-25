use mlua::{Function, Lua};

use crate::lua::Scope;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, name: String| Ok(Scope::building(lua)?.resolve(&name).is_some()))
}
