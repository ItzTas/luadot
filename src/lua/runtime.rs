use std::path::Path;

use mlua::{Lua, Table, Value};

use super::bundled::lpeg;
use super::constants::MODULES_DIR;

pub fn runtime() -> mlua::Result<Lua> {
    let lua = Lua::new();
    lpeg::preload(&lua)?;

    Ok(lua)
}

pub fn environment(lua: &Lua, vars: Option<Table>) -> mlua::Result<Table> {
    let environment = lua.create_table()?;
    if let Some(vars) = vars {
        for pair in vars.pairs::<Value, Value>() {
            let (name, value) = pair?;
            environment.set(name, value)?;
        }
    }

    let meta = lua.create_table()?;
    meta.set("__index", lua.globals())?;
    environment.set_metatable(Some(meta))?;

    Ok(environment)
}

pub fn add_module_path(lua: &Lua, dir: &Path) -> mlua::Result<()> {
    let modules = dir.join(MODULES_DIR);
    let modules = modules.display();
    let package: Table = lua.globals().get("package")?;
    let current: String = package.get("path")?;

    package.set(
        "path",
        format!("{modules}/?.lua;{modules}/?/init.lua;{current}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_layers_the_vars_over_the_globals() {
        let lua = runtime().unwrap();
        let vars = lua.create_table().unwrap();
        vars.set("name", "laptop").unwrap();

        let environment = environment(&lua, Some(vars)).unwrap();
        let value: String = lua
            .load("return type(tostring) .. \"/\" .. name")
            .set_environment(environment)
            .eval()
            .unwrap();

        assert_eq!(value, "function/laptop");
    }

    #[test]
    fn add_module_path_prepends_the_modules_dir() {
        let lua = runtime().unwrap();
        add_module_path(&lua, Path::new("/repo/.config/luadot")).unwrap();

        let package: Table = lua.globals().get("package").unwrap();
        let path: String = package.get("path").unwrap();
        assert!(path.starts_with("/repo/.config/luadot/lua/?.lua;"));
    }
}
