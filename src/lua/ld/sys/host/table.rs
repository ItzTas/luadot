use std::env;

use mlua::{Lua, Table};

use crate::utils;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let host = lua.create_table()?;
    host.set("name", utils::host_name())?;
    host.set("os", env::consts::OS)?;
    host.set("arch", env::consts::ARCH)?;

    Ok(host)
}
