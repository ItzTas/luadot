use std::path::PathBuf;

use mlua::{Function, Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::super::value::text;
use super::constants::{IDENTITY, NAMESPACE};
use crate::lua::Config;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| set(lua, value))
}

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{IDENTITY}"), Surface::Config) {
        return Ok(());
    }

    let raw = text(NAMESPACE, &value, IDENTITY)?;
    if raw.trim().is_empty() {
        return Err(external(format!(
            "`{API}.{NAMESPACE}.{IDENTITY}` takes a path, got an empty string"
        )));
    }

    Config::building(lua)?.set_crypt_identity(PathBuf::from(raw));
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::lua::from_source;

    #[test]
    fn defaults_to_the_backend_keyring() {
        let config = from_source("local unused = 1").unwrap();

        assert_eq!(config.crypt_identity(), None);
    }

    #[test]
    fn takes_the_path_as_it_is_written() {
        let config = from_source(r#"ld.crypt.identity("~/.keys/age.txt")"#).unwrap();

        assert_eq!(config.crypt_identity(), Some(Path::new("~/.keys/age.txt")));
    }

    #[test]
    fn rejects_an_empty_path() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.crypt.identity("  ")"#).unwrap_err()
        );

        assert!(err.contains("`ld.crypt.identity` takes a path, got an empty string"));
    }

    #[test]
    fn rejects_a_value_that_is_not_a_string() {
        let err = format!("{:#}", from_source("ld.crypt.identity(42)").unwrap_err());

        assert!(err.contains("`ld.crypt.identity` takes a string"));
    }
}
