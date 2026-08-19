use mlua::{Lua, Table};

use super::{program, shell};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let cmd = lua.create_table()?;

    let meta = lua.create_table()?;
    meta.set("__call", shell::function(lua)?)?;
    meta.set("__index", program::function(lua)?)?;
    cmd.set_metatable(Some(meta))?;

    Ok(cmd)
}
