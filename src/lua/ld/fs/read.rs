use mlua::{Function, Lua, Value};

use super::constants::READ;
use super::resolve::{failed, resolve};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| {
        let path = resolve(lua, &value, READ)?;
        let bytes =
            std::fs::read(&path).map_err(|err| failed(READ, "read", path.display(), err))?;

        lua.create_string(&bytes)
    })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn yields_what_the_file_holds() {
        let home = tempfile::tempdir().unwrap();
        std::fs::write(home.path().join("lock.json"), "{ \"v\": 1 }\n").unwrap();

        assert_eq!(
            eval(home.path(), r#"return fs.read("lock.json")"#).unwrap(),
            "{ \"v\": 1 }\n"
        );
    }

    #[test]
    fn a_missing_file_is_reported() {
        let home = tempfile::tempdir().unwrap();

        let err = eval(home.path(), r#"return fs.read("missing")"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.fs.read` failed to read"));
    }
}
