use std::path::Path;

use mlua::{Lua, Table};

use super::constants::{CONFIG, DATA, DIR, HOME, REPO};
use super::types::Paths;

pub fn table(lua: &Lua, paths: &Paths) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set(HOME, display(paths.home()))?;
    table.set(CONFIG, display(paths.config()))?;
    table.set(DATA, display(paths.data()))?;

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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn the_data_directory_is_answered() {
        let lua = runtime().unwrap();
        let paths = Paths::new(
            Path::new("/home/u"),
            Path::new("/home/u/.config/luadot"),
            Path::new("/home/u/.local/share/luadot"),
        );

        let table = table(&lua, &paths).unwrap();

        assert_eq!(
            table.get::<String>("data").unwrap(),
            "/home/u/.local/share/luadot"
        );
        assert_eq!(table.get::<Option<String>>("repo").unwrap(), None);
    }
}
