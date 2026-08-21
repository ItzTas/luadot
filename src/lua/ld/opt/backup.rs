use mlua::{Lua, Value};

use super::super::value::flag;
use super::constants::{BACKUP, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, BACKUP)?;
    Config::building(lua, |config| config.set_backup(enabled))?;
    Ok(())
}
