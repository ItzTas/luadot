use mlua::{Function, Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::constants::{BACKUP, BACKUP_KEEP, NAMESPACE};
use super::value::count;
use crate::lua::Config;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| set(lua, value))
}

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{BACKUP_KEEP}"), Surface::Config) {
        return Ok(());
    }

    let keep = count(&value, BACKUP_KEEP)?;
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
    fn every_backup_is_kept_until_a_limit_is_set() {
        let config = from_source("local unused = 1").unwrap();

        assert_eq!(config.backup_keep(), None);
    }

    #[test]
    fn takes_the_number_of_backups_to_keep() {
        let config = from_source("ld.opt.backup_keep(3)").unwrap();

        assert_eq!(config.backup_keep(), Some(3));
    }

    #[test]
    fn the_last_limit_wins() {
        let config = from_source(
            r#"
            ld.opt.backup_keep(3)
            ld.opt.backup_keep(10)
            "#,
        )
        .unwrap();

        assert_eq!(config.backup_keep(), Some(10));
    }

    #[test]
    fn rejects_keeping_nothing() {
        let err = format!("{:#}", from_source("ld.opt.backup_keep(0)").unwrap_err());

        assert!(err.contains("`ld.opt.backup_keep` takes one or more"));
        assert!(err.contains("`ld.opt.backup(false)`"));
    }

    #[test]
    fn rejects_a_value_that_is_not_a_whole_number() {
        let err = format!("{:#}", from_source("ld.opt.backup_keep(2.5)").unwrap_err());

        assert!(err.contains("`ld.opt.backup_keep` takes a whole number"));
    }
}
