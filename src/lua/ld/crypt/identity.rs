use mlua::{Lua, Value};

use super::super::surface::{self, Surface};
use super::super::value::path;
use super::constants::{IDENTITY, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{IDENTITY}"), Surface::Config) {
        return Ok(());
    }

    let identity = path(NAMESPACE, &value, IDENTITY, "a path")?;
    Config::building(lua)?.set_crypt_identity(identity);
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
}
