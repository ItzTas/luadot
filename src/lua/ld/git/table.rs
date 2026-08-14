use mlua::{Lua, Table};

use super::super::path::Paths;
use super::super::table::{Builder, build};
use super::constants::{CONFLICT, IGNORE};
use super::{conflict, ignore, run};

pub fn table(lua: &Lua, paths: &Paths) -> mlua::Result<Table> {
    let functions: [(&str, Builder); 2] =
        [(IGNORE, ignore::function), (CONFLICT, conflict::function)];

    let git = build(lua, &functions)?;

    let meta = lua.create_table()?;
    meta.set("__call", run::function(lua, paths)?)?;
    git.set_metatable(Some(meta))?;

    Ok(git)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::super::super::path::Paths;
    use super::table;
    use crate::lua::runtime::runtime;

    #[test]
    fn the_namespace_is_callable_and_carries_its_functions() {
        let lua = runtime().unwrap();
        let paths = Paths::new(Path::new("/home/u"), Path::new("/home/u/.config/luadot"));
        lua.globals()
            .set("git", table(&lua, &paths).unwrap())
            .unwrap();

        lua.load(
            r#"
            local meta = getmetatable(git)
            assert(type(meta.__call) == "function", "git is not callable")
            assert(type(git.ignore) == "function", "git.ignore is missing")
            assert(type(git.conflict) == "function", "git.conflict is missing")
            "#,
        )
        .exec()
        .unwrap();
    }
}
