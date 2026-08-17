use mlua::{Function, Lua, Value};

use super::super::constants::CRYPT_BACKENDS;
use super::super::parse::lookup;
use super::super::surface::{self, Surface};
use super::constants::{BACKEND, NAMESPACE};
use super::value::text;
use crate::lua::Config;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| set(lua, value))
}

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{BACKEND}"), Surface::Config) {
        return Ok(());
    }

    let name = text(&value, BACKEND)?;
    let backend = lookup(&CRYPT_BACKENDS, &name, "crypt backend")?;
    Config::building(lua)?.set_crypt_backend(backend);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::crypt::Backend;
    use crate::lua::from_source;

    #[test]
    fn defaults_to_age() {
        let config = from_source("local unused = 1").unwrap();

        assert_eq!(config.crypt_backend(), Backend::Age);
    }

    #[test]
    fn sets_the_backend() {
        let config = from_source(r#"ld.crypt.backend("gpg")"#).unwrap();

        assert_eq!(config.crypt_backend(), Backend::Gpg);
    }

    #[test]
    fn rejects_an_unknown_backend() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.crypt.backend("vault")"#).unwrap_err()
        );

        assert!(err.contains("unknown crypt backend `vault`"));
        assert!(err.contains("age, gpg"));
    }

    #[test]
    fn rejects_a_value_that_is_not_a_string() {
        let err = format!("{:#}", from_source("ld.crypt.backend(true)").unwrap_err());

        assert!(err.contains("`ld.crypt.backend` takes a string"));
    }
}
