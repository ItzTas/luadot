use mlua::{Lua, Value};

use super::super::surface::{self, Surface};
use super::super::value::path;
use super::constants::{NAMESPACE, REPO_DIR};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{REPO_DIR}"), Surface::Config) {
        return Ok(());
    }

    let dir = path(NAMESPACE, &value, REPO_DIR, "a directory")?;
    Config::building(lua)?.set_repo_dir(dir);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::lua::from_source;

    #[test]
    fn takes_the_directory_as_it_is_written() {
        let config = from_source(r#"ld.opt.repo_dir("~/dotfiles")"#).unwrap();

        assert_eq!(config.repo_dir(), Some(Path::new("~/dotfiles")));
    }
}
