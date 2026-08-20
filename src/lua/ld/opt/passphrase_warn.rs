use mlua::{Lua, Value};

use super::super::value::flag;
use super::constants::{NAMESPACE, PASSPHRASE_WARN};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, PASSPHRASE_WARN)?;
    Config::building(lua)?.set_passphrase_warn(enabled);
    Ok(())
}
