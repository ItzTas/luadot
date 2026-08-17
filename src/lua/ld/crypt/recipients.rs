use mlua::{Function, Lua, Value};

use super::super::surface::{self, Surface};
use super::constants::{NAMESPACE, RECIPIENTS};
use super::value::keys;
use crate::lua::Config;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| set(lua, value))
}

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{RECIPIENTS}"), Surface::Config) {
        return Ok(());
    }

    let recipients = keys(&value, RECIPIENTS)?;
    Config::building(lua)?.set_crypt_recipients(recipients);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua::from_source;

    #[test]
    fn defaults_to_no_recipients() {
        let config = from_source("local unused = 1").unwrap();

        assert!(config.crypt_recipients().is_empty());
    }

    #[test]
    fn takes_a_single_key() {
        let config = from_source(r#"ld.crypt.recipients("age1example")"#).unwrap();

        assert_eq!(config.crypt_recipients(), ["age1example"]);
    }

    #[test]
    fn takes_a_list_of_keys() {
        let config =
            from_source(r#"ld.crypt.recipients({ "age1first", "age1second" })"#).unwrap();

        assert_eq!(config.crypt_recipients(), ["age1first", "age1second"]);
    }

    #[test]
    fn the_last_call_wins() {
        let config = from_source(
            r#"
            ld.crypt.recipients("age1old")
            ld.crypt.recipients({ "age1new" })
            "#,
        )
        .unwrap();

        assert_eq!(config.crypt_recipients(), ["age1new"]);
    }

    #[test]
    fn rejects_an_empty_list() {
        let err = format!(
            "{:#}",
            from_source("ld.crypt.recipients({})").unwrap_err()
        );

        assert!(err.contains("`ld.crypt.recipients` takes a key or a list of keys"));
    }

    #[test]
    fn rejects_a_value_that_is_not_a_key() {
        assert!(from_source("ld.crypt.recipients(42)").is_err());
    }
}
