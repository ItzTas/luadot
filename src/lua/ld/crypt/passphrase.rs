use mlua::{Lua, Value};

use super::super::surface::{self, Surface};
use super::super::value::flag;
use super::constants::{NAMESPACE, PASSPHRASE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{PASSPHRASE}"), Surface::Config) {
        return Ok(());
    }

    let passphrase = flag(NAMESPACE, &value, PASSPHRASE)?;
    Config::building(lua)?.set_crypt_passphrase(passphrase);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua::from_source;

    #[test]
    fn defaults_to_the_recipient_keys() {
        let config = from_source("local unused = 1").unwrap();

        assert!(!config.crypt_passphrase());
    }

    #[test]
    fn turns_the_passphrase_mode_on() {
        let config = from_source("ld.crypt.passphrase(true)").unwrap();

        assert!(config.crypt_passphrase());
    }

    #[test]
    fn the_last_call_wins() {
        let config = from_source(
            r#"
            ld.crypt.passphrase(true)
            ld.crypt.passphrase(false)
            "#,
        )
        .unwrap();

        assert!(!config.crypt_passphrase());
    }

    #[test]
    fn rejects_a_value_that_is_not_a_boolean() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.crypt.passphrase("hunter2")"#).unwrap_err()
        );

        assert!(err.contains("`ld.crypt.passphrase` takes true or false"));
    }
}
