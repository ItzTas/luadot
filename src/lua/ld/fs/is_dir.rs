use mlua::{Function, Lua, Value};

use super::constants::IS_DIR;
use super::resolve::resolve;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| Ok(resolve(lua, &value, IS_DIR)?.is_dir()))
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn only_a_directory_answers_true() {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir(home.path().join("plugins")).unwrap();
        std::fs::write(home.path().join("lock.json"), "{}").unwrap();

        assert_eq!(
            eval(
                home.path(),
                r#"return tostring(fs.is_dir("plugins")) .. "/" .. tostring(fs.is_dir("lock.json")) .. "/" .. tostring(fs.is_dir("missing"))"#
            )
            .unwrap(),
            "true/false/false"
        );
    }
}
