use mlua::{Lua, Value};

use super::super::value::path;
use super::constants::{BACKUP_DIR, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let dir = path(NAMESPACE, &value, BACKUP_DIR, "a directory")?;
    Config::building(lua, |config| config.set_backup_dir(dir))?;
    Ok(())
}
