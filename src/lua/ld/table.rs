use mlua::{Function, Lua, Table};

pub type Builder = fn(&Lua) -> mlua::Result<Function>;

pub fn build(lua: &Lua, functions: &[(&str, Builder)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (name, function) in functions {
        table.set(*name, function(lua)?)?;
    }
    Ok(table)
}
