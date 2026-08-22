use std::path::Path;
use std::sync::Mutex;

use super::super::path::Paths;
use super::super::surface::Surface;
use super::constants::NAMESPACE;
use super::table::table;
use crate::lua::runtime::runtime;
use crate::lua::{Config, Scope, Shared};

pub fn eval(home: &Path, source: &str) -> mlua::Result<String> {
    let lua = runtime().unwrap();
    let paths = Paths::new(
        home,
        &home.join(".config/luadot"),
        &home.join(".local/share/luadot"),
    );
    Surface::Exec.install(&lua);
    lua.set_app_data(Shared::new(Mutex::new(Config::default())));
    lua.set_app_data(Scope::new(
        paths.config().to_path_buf(),
        paths.home().to_path_buf(),
    ));
    lua.globals()
        .set(NAMESPACE, table(&lua, &paths).unwrap())
        .unwrap();

    lua.load(source).eval()
}
