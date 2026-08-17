use mlua::{Function, Lua, Table, Value};

use super::constants::API;
use super::parse::{external, lookup};

pub type Builder = fn(&Lua) -> mlua::Result<Function>;

pub type Setter = fn(&Lua, Value) -> mlua::Result<()>;

pub fn build(lua: &Lua, functions: &[(&str, Builder)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (name, function) in functions {
        table.set(*name, function(lua)?)?;
    }
    Ok(table)
}

pub fn options(
    lua: &Lua,
    namespace: &'static str,
    entries: &'static [(&'static str, Setter)],
    field: &'static str,
) -> mlua::Result<Table> {
    let table = setters(lua, entries)?;
    let meta = lua.create_table()?;
    meta.set(
        "__call",
        lua.create_function(move |lua, (_, options): (Table, Table)| {
            apply(lua, &options, namespace, entries, field)
        })?,
    )?;
    table.set_metatable(Some(meta))?;

    Ok(table)
}

fn setters(lua: &Lua, setters: &[(&str, Setter)]) -> mlua::Result<Table> {
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

fn apply(
    lua: &Lua,
    options: &Table,
    namespace: &str,
    entries: &[(&str, Setter)],
    field: &str,
) -> mlua::Result<()> {
    for pair in options.clone().pairs::<String, Value>() {
        let (name, value) =
            pair.map_err(|_| external(format!("`{API}.{namespace}` takes a table of options")))?;
        lookup(entries, &name, field)?(lua, value)?;
    }
    Ok(())
}
