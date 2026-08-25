use mlua::{Lua, Value};

use super::super::value::flag;
use super::constants::{AUTOCOMMIT, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, AUTOCOMMIT)?;
    Config::building(lua, |config| config.set_autocommit(enabled))?;
    Ok(())
}
