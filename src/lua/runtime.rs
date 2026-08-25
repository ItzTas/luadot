use std::path::{Path, PathBuf};

use mlua::{Lua, Table, Value};

use super::bundled::lpeg;
use super::constants::{MODULE_PATH_MISSING, MODULES_DIR, PACKAGE, PACKAGE_PATH};

#[derive(Debug, Clone)]
struct ModulePath {
    own: Vec<PathBuf>,
    registered: Vec<PathBuf>,
    default: String,
}

pub fn runtime() -> mlua::Result<Lua> {
    let lua = Lua::new();
    lpeg::preload(&lua)?;

    let package: Table = lua.globals().get(PACKAGE)?;
    lua.set_app_data(ModulePath {
        own: Vec::new(),
        registered: Vec::new(),
        default: package.get(PACKAGE_PATH)?,
    });

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
    change(lua, |path| path.own.insert(0, dir.to_path_buf()))
}

pub fn register_module_path(lua: &Lua, dir: &Path) -> mlua::Result<()> {
    change(lua, |path| {
        if !path.registered.iter().any(|registered| registered == dir) {
            path.registered.push(dir.to_path_buf());
        }
    })
}

fn change(lua: &Lua, edit: impl FnOnce(&mut ModulePath)) -> mlua::Result<()> {
    let rendered = {
        let mut path = lua
            .app_data_mut::<ModulePath>()
            .ok_or_else(|| mlua::Error::external(MODULE_PATH_MISSING))?;
        edit(&mut path);
        path.render()
    };

    let package: Table = lua.globals().get(PACKAGE)?;
    package.set(PACKAGE_PATH, rendered)
}

impl ModulePath {
    fn render(&self) -> String {
        self.own
            .iter()
            .chain(&self.registered)
            .map(|dir| entries(dir))
            .chain([self.default.clone()])
            .collect::<Vec<_>>()
            .join(";")
    }
}

fn entries(dir: &Path) -> String {
    let modules = dir.join(MODULES_DIR);
    let modules = modules.display();

    format!("{modules}/?.lua;{modules}/?/init.lua")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(lua: &Lua) -> String {
        let package: Table = lua.globals().get(PACKAGE).unwrap();
        package.get(PACKAGE_PATH).unwrap()
    }

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
    fn the_own_directories_come_first_and_the_registered_ones_follow_in_order() {
        let lua = runtime().unwrap();
        let default = path(&lua);

        register_module_path(&lua, Path::new("/plugins/first")).unwrap();
        register_module_path(&lua, Path::new("/plugins/second")).unwrap();
        register_module_path(&lua, Path::new("/plugins/first")).unwrap();
        add_module_path(&lua, Path::new("/repo/.config/luadot")).unwrap();

        assert_eq!(
            path(&lua),
            format!(
                "/repo/.config/luadot/lua/?.lua;/repo/.config/luadot/lua/?/init.lua;\
                 /plugins/first/lua/?.lua;/plugins/first/lua/?/init.lua;\
                 /plugins/second/lua/?.lua;/plugins/second/lua/?/init.lua;{default}"
            )
        );
    }
}
