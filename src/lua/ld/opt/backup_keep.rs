use mlua::{Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::super::value::count;
use super::constants::{BACKUP, BACKUP_KEEP, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{BACKUP_KEEP}"), Surface::Config) {
        return Ok(());
    }

    let keep = count(NAMESPACE, &value, BACKUP_KEEP)?;
    if keep == 0 {
        return Err(external(format!(
            "`{API}.{NAMESPACE}.{BACKUP_KEEP}` takes one or more; `{API}.{NAMESPACE}.{BACKUP}(false)` is how backups are turned off"
        )));
    }

    Config::building(lua)?.set_backup_keep(keep);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua::from_source;

    #[test]
    fn rejects_keeping_nothing() {
        let err = format!("{:#}", from_source("ld.opt.backup_keep(0)").unwrap_err());

        assert!(err.contains("`ld.opt.backup_keep` takes one or more"));
        assert!(err.contains("`ld.opt.backup(false)`"));
    }
}
