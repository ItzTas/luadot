use std::path::Path;

use mlua::{Lua, Table};

use super::constants::{CONFIG, DIR, HOME, REPO};
use super::types::Paths;

pub fn table(lua: &Lua, paths: &Paths) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set(HOME, display(paths.home()))?;
    table.set(CONFIG, display(paths.config()))?;

    for (name, path) in [(REPO, paths.repo()), (DIR, paths.dir())] {
        let Some(path) = path else {
            continue;
        };
        table.set(name, display(path))?;
    }

    Ok(table)
}

fn display(path: &Path) -> String {
    path.display().to_string()
}
