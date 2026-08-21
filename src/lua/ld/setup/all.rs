use mlua::{Function, Lua, Table};

use super::super::class;
use super::super::constants::API;
use super::super::parse::{chain, external};
use super::super::path::Paths;
use super::super::surface;
use super::constants::{ALL, NAMESPACE};
use super::scripts::{listing, run};
use crate::lua::Config;
use crate::lua::setup;

pub fn function(lua: &Lua, paths: &Paths) -> mlua::Result<Function> {
    let paths = paths.clone();
    let command = format!("`{API}.{NAMESPACE}.{ALL}`");

    lua.create_function(move |lua, options: Option<Table>| {
        surface::slow(lua, &format!("{NAMESPACE}.{ALL}"));

        let order = order(&options)?;
        let (repo, names) = listing(&paths, &command)?;
        let classes = class::current(lua);
        let shared = Config::shared(lua)?;

        for name in setup::ordered(&command, names, &order).map_err(chain)? {
            run(&paths, &command, repo, &name, &classes, &shared)?;
        }
        Ok(())
    })
}

fn order(options: &Option<Table>) -> mlua::Result<Vec<String>> {
    let Some(options) = options else {
        return Ok(Vec::new());
    };

    options
        .get::<Option<Vec<String>>>("order")
        .map(Option::unwrap_or_default)
        .map_err(|_| {
            external(format!(
                "`{API}.{NAMESPACE}.{ALL}` takes a table with an optional `order` list of setup names"
            ))
        })
}
