use mlua::{Lua, Table};

use super::super::path::Paths;
use super::run;

pub fn table(lua: &Lua, paths: &Paths) -> mlua::Result<Table> {
    let git = lua.create_table()?;

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
    fn the_namespace_is_callable_and_carries_no_function() {
        let lua = runtime().unwrap();
        let paths = Paths::new(Path::new("/home/u"), Path::new("/home/u/.config/luadot"));
        lua.globals()
            .set("git", table(&lua, &paths).unwrap())
            .unwrap();

        lua.load(
            r#"
            local meta = getmetatable(git)
            assert(type(meta.__call) == "function", "git is not callable")
            assert(git.conflict == nil, "git.conflict outlived the move to ld.opt")
            assert(git.ignore == nil, "git.ignore outlived the move to ld.rules")
            "#,
        )
        .exec()
        .unwrap();
    }
}
