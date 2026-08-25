use mlua::{Function, Lua, Value};

use super::constants::EXISTS;
use super::resolve::resolve;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| {
        let path = resolve(lua, &value, EXISTS)?;

        Ok(std::fs::symlink_metadata(path).is_ok())
    })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn a_dangling_symlink_is_there_and_a_missing_path_is_not() {
        let home = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink("/nowhere", home.path().join("link")).unwrap();

        assert_eq!(
            eval(
                home.path(),
                r#"return tostring(fs.exists("link")) .. "/" .. tostring(fs.exists("missing"))"#
            )
            .unwrap(),
            "true/false"
        );
    }
}
