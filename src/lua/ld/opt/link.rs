use mlua::{Lua, Value};

use super::super::constants::LINK_MODES;
use super::super::value::choice;
use super::constants::{LINK, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let mode = choice(NAMESPACE, &value, LINK, &LINK_MODES, "link mode")?;
    Config::building(lua, |config| config.set_link(mode))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua::from_source;

    #[test]
    fn rejects_an_unknown_link_mode() {
        let err = format!("{:#}", from_source(r#"ld.opt.link("magic")"#).unwrap_err());

        assert!(err.contains("unknown link mode `magic`"));
        assert!(err.contains("hard, symbolic, copy"));
    }
}
