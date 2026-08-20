use mlua::{Lua, Value};

use super::super::value::flag;
use super::constants::{NAMESPACE, PKG_WARN};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, PKG_WARN)?;
    Config::building(lua)?.set_pkg_warn(enabled);
    Ok(())
}
