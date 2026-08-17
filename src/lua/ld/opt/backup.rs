use mlua::{Lua, Value};

use super::super::surface::{self, Surface};
use super::super::value::flag;
use super::constants::{BACKUP, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{BACKUP}"), Surface::Config) {
        return Ok(());
    }

    let enabled = flag(NAMESPACE, &value, BACKUP)?;
    Config::building(lua)?.set_backup(enabled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua::from_source;

    #[test]
    fn defaults_to_backing_up() {
        let config = from_source("local unused = 1").unwrap();

        assert!(config.backup());
    }

    #[test]
    fn turns_the_backup_off() {
        let config = from_source("ld.opt.backup(false)").unwrap();

        assert!(!config.backup());
    }

    #[test]
    fn turns_the_backup_back_on() {
        let config = from_source(
            r#"
            ld.opt.backup(false)
            ld.opt.backup(true)
            "#,
        )
        .unwrap();

        assert!(config.backup());
    }

    #[test]
    fn rejects_a_value_that_is_not_a_boolean() {
        let err = format!("{:#}", from_source(r#"ld.opt.backup("off")"#).unwrap_err());

        assert!(err.contains("`ld.opt.backup` takes true or false"));
    }
}
