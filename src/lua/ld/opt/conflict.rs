use mlua::{Lua, Value};

use super::super::constants::CONFLICT_POLICIES;
use super::super::value::choice;
use super::constants::{CONFLICT, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let policy = choice(
        NAMESPACE,
        &value,
        CONFLICT,
        &CONFLICT_POLICIES,
        "conflict policy",
    )?;
    Config::building(lua, |config| config.set_conflict(policy))?;
    Ok(())
}
