use std::path::PathBuf;

use mlua::Lua;

use super::super::parse::external;
use crate::lua::Config;
use crate::lua::config::constants::LOCKED;
use crate::lua::runtime::register_module_path;

pub fn extend(lua: &Lua) -> mlua::Result<()> {
    let registered: Vec<PathBuf> = {
        let shared = Config::shared(lua)?;
        let config = shared.try_lock().map_err(|_| external(LOCKED))?;
        config.runtime_paths().to_vec()
    };

    for dir in registered {
        register_module_path(lua, &dir)?;
    }

    Ok(())
}
