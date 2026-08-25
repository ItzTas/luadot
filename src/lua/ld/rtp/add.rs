use mlua::{Function, Lua, Value};

use super::super::value::path;
use super::constants::{ADD, NAMESPACE};
use crate::lua::runtime::register_module_path;
use crate::lua::{Config, Scope};
use crate::utils::expand;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| {
        let dir = path(NAMESPACE, &value, ADD, "a directory")?;
        let home = Scope::building(lua)?.home().to_path_buf();
        let dir = expand(&home, &dir);

        register_module_path(lua, &dir)?;
        Config::building(lua, |config| config.add_runtime_path(dir))
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::super::fixture::plugin;
    use crate::files::LinkMode;
    use crate::lua::from_source;

    #[test]
    fn a_registered_directory_is_requirable_at_once_and_remembered() {
        let root = tempfile::tempdir().unwrap();
        let dir = plugin(root.path(), "links", r#"ld.opt.link("symbolic")"#);

        let config = from_source(&format!(
            r#"
            ld.rtp.add("{}")
            ld.rtp.add("{}")
            require("links")
            "#,
            dir.display(),
            dir.display()
        ))
        .unwrap();

        assert_eq!(config.link_mode(Path::new(".bashrc")), LinkMode::Symbolic);
        assert_eq!(config.runtime_paths(), [dir]);
    }

    #[test]
    fn rejects_anything_but_a_directory_name() {
        let err = format!("{:#}", from_source("ld.rtp.add(true)").unwrap_err());

        assert!(err.contains("`ld.rtp.add` takes a string"));
    }
}
