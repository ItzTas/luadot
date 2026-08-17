use std::path::PathBuf;

use mlua::{Function, Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::surface::{self, Surface};
use super::super::value::text;
use super::constants::{NAMESPACE, REPO_DIR};
use crate::lua::Config;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| set(lua, value))
}

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{REPO_DIR}"), Surface::Config) {
        return Ok(());
    }

    let raw = text(NAMESPACE, &value, REPO_DIR)?;
    if raw.trim().is_empty() {
        return Err(external(format!(
            "`{API}.{NAMESPACE}.{REPO_DIR}` takes a directory, got an empty string"
        )));
    }

    Config::building(lua)?.set_repo_dir(PathBuf::from(raw));
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::lua::from_source;

    #[test]
    fn defaults_to_the_repository_clone_left_behind() {
        let config = from_source("local unused = 1").unwrap();

        assert_eq!(config.repo_dir(), None);
    }

    #[test]
    fn takes_the_directory_as_it_is_written() {
        let config = from_source(r#"ld.opt.repo_dir("~/dotfiles")"#).unwrap();

        assert_eq!(config.repo_dir(), Some(Path::new("~/dotfiles")));
    }

    #[test]
    fn the_last_directory_wins() {
        let config = from_source(
            r#"
            ld.opt.repo_dir("~/first")
            ld.opt.repo_dir("/data/second")
            "#,
        )
        .unwrap();

        assert_eq!(config.repo_dir(), Some(Path::new("/data/second")));
    }

    #[test]
    fn rejects_a_value_that_is_not_a_string() {
        let err = format!("{:#}", from_source("ld.opt.repo_dir(true)").unwrap_err());

        assert!(err.contains("`ld.opt.repo_dir` takes a string"));
    }

    #[test]
    fn rejects_an_empty_directory() {
        let err = format!("{:#}", from_source(r#"ld.opt.repo_dir("")"#).unwrap_err());

        assert!(err.contains("`ld.opt.repo_dir` takes a directory"));
    }
}
