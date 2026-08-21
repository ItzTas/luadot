use mlua::{Lua, Value};

use super::super::value::flag;
use super::constants::{AUTOPUSH, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, AUTOPUSH)?;
    Config::building(lua, |config| config.set_autopush(enabled))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::lua::from_source;

    #[test]
    fn pushing_on_its_own_commits_first() {
        let config = from_source("ld.opt.autopush(true)").unwrap();

        assert!(config.autopush(Path::new("home/.bashrc")));
        assert!(config.autocommit(Path::new("home/.bashrc")));
    }
}
