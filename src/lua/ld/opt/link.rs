use mlua::{Lua, Value};

use super::super::constants::LINK_MODES;
use super::super::surface::{self, Surface};
use super::super::value::choice;
use super::constants::{LINK, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{LINK}"), Surface::Config) {
        return Ok(());
    }

    let mode = choice(NAMESPACE, &value, LINK, &LINK_MODES, "link mode")?;
    Config::building(lua)?.set_link(mode);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::files::LinkMode;
    use crate::lua::from_source;

    #[test]
    fn sets_the_default_link_mode() {
        let config = from_source(r#"ld.opt.link("symbolic")"#).unwrap();

        assert_eq!(config.link_mode(Path::new(".bashrc")), LinkMode::Symbolic);
    }

    #[test]
    fn rejects_an_unknown_link_mode() {
        let err = format!("{:#}", from_source(r#"ld.opt.link("magic")"#).unwrap_err());

        assert!(err.contains("unknown link mode `magic`"));
        assert!(err.contains("hard, symbolic, copy"));
    }

    #[test]
    fn rejects_a_value_that_is_not_a_string() {
        let err = format!("{:#}", from_source("ld.opt.link(true)").unwrap_err());

        assert!(err.contains("`ld.opt.link` takes a string"));
    }
}
