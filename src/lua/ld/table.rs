use mlua::{Function, Lua, Table, Value};

pub type Builder = fn(&Lua) -> mlua::Result<Function>;

pub type Setter = fn(&Lua, Value) -> mlua::Result<()>;

pub fn build(lua: &Lua, functions: &[(&str, Builder)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (name, function) in functions {
        table.set(*name, function(lua)?)?;
    }
    Ok(table)
}

pub fn setters(lua: &Lua, setters: &[(&str, Setter)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (name, setter) in setters {
        let setter = *setter;
        table.set(
            *name,
            lua.create_function(move |lua, value: Value| setter(lua, value))?,
        )?;
    }
    Ok(table)
}
