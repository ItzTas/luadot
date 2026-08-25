use mlua::{Function, Lua};

use super::super::json::encoder;
use super::constants::{JSON, NAMESPACE};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    encoder(lua, &format!("{NAMESPACE}.{JSON}"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::lua::ld::{Paths, Surface, install};
    use crate::lua::runtime::runtime;
    use crate::state::Classes;

    fn json(source: &str) -> mlua::Result<String> {
        let lua = runtime().unwrap();
        let paths = Paths::new(
            Path::new("/home/u"),
            Path::new("/home/u/.config/luadot"),
            Path::new("/home/u/.local/share/luadot"),
        );
        install(&lua, Surface::Standalone, &paths, &Classes::default()).unwrap();

        lua.load(source).eval()
    }

    fn error(source: &str) -> String {
        json(source).unwrap_err().to_string()
    }

    #[test]
    fn a_named_table_sorts_its_keys() {
        assert_eq!(
            json(r#"return ld.alt.json({ editor = "nvim", gpu = "amd" })"#).unwrap(),
            "{\n  \"editor\": \"nvim\",\n  \"gpu\": \"amd\"\n}"
        );
    }

    #[test]
    fn every_scalar_keeps_its_type() {
        assert_eq!(json("return ld.alt.json(true)").unwrap(), "true");
        assert_eq!(json("return ld.alt.json(2)").unwrap(), "2");
        assert_eq!(json("return ld.alt.json(2.5)").unwrap(), "2.5");
        assert_eq!(json(r#"return ld.alt.json("text")"#).unwrap(), "\"text\"");
        assert_eq!(json("return ld.alt.json(nil)").unwrap(), "null");
    }

    #[test]
    fn a_table_holding_itself_is_reported() {
        let err = error("local t = {}; t.self = t; return ld.alt.json(t)");

        assert!(err.contains("`ld.alt.json` gave up"));
        assert!(err.contains("a table holding itself never ends"));
    }
}
