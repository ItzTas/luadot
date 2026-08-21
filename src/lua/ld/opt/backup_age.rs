use mlua::{Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::value::span;
use super::constants::{BACKUP, BACKUP_AGE, NAMESPACE, SPAN_KIND};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let age = span(NAMESPACE, &value, BACKUP_AGE, SPAN_KIND)?;
    if age == 0 {
        return Err(external(format!(
            "`{API}.{NAMESPACE}.{BACKUP_AGE}` takes one second or more; `{API}.{NAMESPACE}.{BACKUP}(false)` is how backups are turned off"
        )));
    }

    Config::building(lua, |config| config.set_backup_age(age))?;
    Ok(())
}
