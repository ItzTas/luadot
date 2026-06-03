mod api;
mod handle;

pub use api::{Host, install_template_api, set_host};
pub use handle::FileHandle;

use mlua::Lua;

pub fn runtime() -> mlua::Result<Lua> {
    Ok(Lua::new())
}
