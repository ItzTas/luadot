use mlua::{Lua, Value};

use super::super::surface::{self, Surface};
use super::super::value::flag;
use super::constants::{NAMESPACE, PASSPHRASE_WARN};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(
        lua,
        &format!("{NAMESPACE}.{PASSPHRASE_WARN}"),
        Surface::Config,
    ) {
        return Ok(());
    }

    let enabled = flag(NAMESPACE, &value, PASSPHRASE_WARN)?;
    Config::building(lua)?.set_crypt_passphrase_warn(enabled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua::from_source;

    #[test]
    fn defaults_to_warning() {
        let config = from_source("local unused = 1").unwrap();

        assert!(config.crypt_passphrase_warn());
    }

    #[test]
    fn turns_the_warning_off() {
        let config = from_source("ld.crypt.passphrase_warn(false)").unwrap();

        assert!(!config.crypt_passphrase_warn());
    }

    #[test]
    fn turns_the_warning_back_on() {
        let config = from_source(
            r#"
            ld.crypt.passphrase_warn(false)
            ld.crypt.passphrase_warn(true)
            "#,
        )
        .unwrap();

        assert!(config.crypt_passphrase_warn());
    }

    #[test]
    fn rejects_a_value_that_is_not_a_boolean() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.crypt.passphrase_warn("off")"#).unwrap_err()
        );

        assert!(err.contains("`ld.crypt.passphrase_warn` takes true or false"));
    }
}
