use std::path::Path;

use mlua::{Lua, Table};

use super::constants::MODULES_DIR;

pub fn runtime() -> mlua::Result<Lua> {
    Ok(Lua::new())
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
    fn runtime_evaluates_expression() {
        let lua = runtime().unwrap();
        let value: i64 = lua.load("return 1 + 2").eval().unwrap();
        assert_eq!(value, 3);
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
