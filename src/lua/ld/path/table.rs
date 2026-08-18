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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn exposes_every_path_as_a_string() {
        let lua = runtime().unwrap();
        let paths = Paths::new(Path::new("/home/u"), Path::new("/home/u/.config/luadot"))
            .with_repo(Some(Path::new("/data/repo")))
            .with_dir(Path::new("/data/repo/.zshrc.luadot"));

        let table = table(&lua, &paths).unwrap();

        assert_eq!(table.get::<String>("home").unwrap(), "/home/u");
        assert_eq!(
            table.get::<String>("config").unwrap(),
            "/home/u/.config/luadot"
        );
        assert_eq!(table.get::<String>("repo").unwrap(), "/data/repo");
        assert_eq!(
            table.get::<String>("dir").unwrap(),
            "/data/repo/.zshrc.luadot"
        );
    }
}
