use mlua::{Lua, Value};

use super::super::value::flag;
use super::constants::{AUTOPUSH, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, AUTOPUSH)?;
    Config::building(lua, |config| config.set_autopush(enabled))?;
    Ok(())
}
