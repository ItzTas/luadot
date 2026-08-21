use std::env;

use mlua::{Lua, Table};

use super::constants::{ARCH, NAME, OS};
use crate::utils;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let host = lua.create_table()?;
    host.set(NAME, utils::host_name())?;
    host.set(OS, env::consts::OS)?;
    host.set(ARCH, env::consts::ARCH)?;

    Ok(host)
}
