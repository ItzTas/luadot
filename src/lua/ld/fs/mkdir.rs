use mlua::{Function, Lua, Value};

use super::constants::MKDIR;
use super::resolve::{failed, resolve};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| {
        let path = resolve(lua, &value, MKDIR)?;

        std::fs::create_dir_all(&path).map_err(|err| failed(MKDIR, "create", path.display(), err))
    })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn creates_every_directory_leading_there_and_accepts_one_already_there() {
        let home = tempfile::tempdir().unwrap();

        eval(
            home.path(),
            r#"
            fs.mkdir("plugins/lazyld/lua")
            fs.mkdir("plugins/lazyld/lua")
            return "done"
            "#,
        )
        .unwrap();

        assert!(home.path().join("plugins/lazyld/lua").is_dir());
    }
}
