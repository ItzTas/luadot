use std::fmt::Display;
use std::path::PathBuf;

use mlua::{Lua, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::super::value::path;
use super::constants::NAMESPACE;
use crate::lua::Scope;
use crate::utils::expand;

pub fn resolve(lua: &Lua, value: &Value, call: &str) -> mlua::Result<PathBuf> {
    let raw = path(NAMESPACE, value, call, "a path")?;
    let home = Scope::building(lua)?.home().to_path_buf();

    Ok(expand(&home, &raw))
}

pub fn failed(call: &str, action: &str, subject: impl Display, err: impl Display) -> mlua::Error {
    external(format!(
        "`{API}.{NAMESPACE}.{call}` failed to {action} {subject}: {err}"
    ))
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn a_tilde_and_a_relative_path_start_at_the_home_directory() {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(home.path().join(".local/share")).unwrap();

        assert_eq!(
            eval(
                home.path(),
                r#"return tostring(fs.is_dir("~/.local")) .. "/" .. tostring(fs.is_dir(".local/share"))"#
            )
            .unwrap(),
            "true/true"
        );
    }

    #[test]
    fn rejects_anything_but_a_path() {
        let home = tempfile::tempdir().unwrap();

        let err = eval(home.path(), "return fs.exists(true)")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.fs.exists` takes a string"));
    }
}
