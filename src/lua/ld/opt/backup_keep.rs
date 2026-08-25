use mlua::{Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::value::count;
use super::constants::{BACKUP, BACKUP_KEEP, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let keep = count(NAMESPACE, &value, BACKUP_KEEP)?;
    if keep == 0 {
        return Err(external(format!(
            "`{API}.{NAMESPACE}.{BACKUP_KEEP}` takes one or more; `{API}.{NAMESPACE}.{BACKUP}(false)` is how backups are turned off"
        )));
    }

    Config::building(lua, |config| config.set_backup_keep(keep))?;
    Ok(())
}
