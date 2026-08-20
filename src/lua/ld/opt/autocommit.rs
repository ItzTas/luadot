use mlua::{Lua, Value};

use super::super::surface::{self, Surface};
use super::super::value::flag;
use super::constants::{AUTOCOMMIT, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{AUTOCOMMIT}"), Surface::Config) {
        return Ok(());
    }

    let enabled = flag(NAMESPACE, &value, AUTOCOMMIT)?;
    Config::building(lua)?.set_autocommit(enabled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::lua::from_source;

    #[test]
    fn defaults_to_committing_nothing() {
        let config = from_source("local unused = 1").unwrap();

        assert!(!config.autocommit(Path::new("home/.bashrc")));
    }

    #[test]
    fn turns_the_commit_on_for_every_file() {
        let config = from_source("ld.opt.autocommit(true)").unwrap();

        assert!(config.autocommit(Path::new("home/.bashrc")));
        assert!(!config.autopush(Path::new("home/.bashrc")));
    }
}
