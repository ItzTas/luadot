use mlua::{Lua, Value};

use super::super::value::flag;
use super::constants::{LFS, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, LFS)?;
    Config::building(lua, |config| config.set_lfs(enabled))?;
    Ok(())
}
