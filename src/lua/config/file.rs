use std::path::Path;

use mlua::{Lua, Table};

use super::constants::{PATH, SIDE, STATE, SYSTEM};
use crate::files::Side;

pub fn table(
    lua: &Lua,
    path: &Path,
    system: &Path,
    side: Side,
    state: &str,
) -> mlua::Result<Table> {
    let file = lua.create_table()?;
    file.set(PATH, path.to_string_lossy().as_ref())?;
    file.set(SYSTEM, system.to_string_lossy().as_ref())?;
    file.set(SIDE, side.dir())?;
    file.set(STATE, state)?;

    Ok(file)
}
