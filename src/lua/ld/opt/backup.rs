use mlua::{Lua, Value};

use super::super::surface::{self, Surface};
use super::super::value::flag;
use super::constants::{BACKUP, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{BACKUP}"), Surface::Config) {
        return Ok(());
    }

    let enabled = flag(NAMESPACE, &value, BACKUP)?;
    Config::building(lua)?.set_backup(enabled);
    Ok(())
}
