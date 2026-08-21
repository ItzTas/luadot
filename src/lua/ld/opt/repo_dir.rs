use mlua::{Lua, Value};

use super::super::value::path;
use super::constants::{NAMESPACE, REPO_DIR};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let dir = path(NAMESPACE, &value, REPO_DIR, "a directory")?;
    Config::building(lua, |config| config.set_repo_dir(dir))?;
    Ok(())
}
