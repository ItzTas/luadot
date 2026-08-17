use mlua::{Lua, Value};

use super::super::value::flag;
use super::constants::{NAMESPACE, PKG_WARN};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let enabled = flag(NAMESPACE, &value, PKG_WARN)?;
    Config::building(lua)?.set_pkg_warn(enabled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua::from_source;

    #[test]
    fn defaults_to_warning() {
        let config = from_source("local unused = 1").unwrap();

        assert!(config.pkg_warn());
    }

    #[test]
    fn turns_the_warning_off() {
        let config = from_source("ld.opt.pkg_warn(false)").unwrap();

        assert!(!config.pkg_warn());
    }

    #[test]
    fn turns_the_warning_back_on() {
        let config = from_source(
            r#"
            ld.opt.pkg_warn(false)
            ld.opt.pkg_warn(true)
            "#,
        )
        .unwrap();

        assert!(config.pkg_warn());
    }

    #[test]
    fn rejects_a_value_that_is_not_a_boolean() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.opt.pkg_warn("off")"#).unwrap_err()
        );

        assert!(err.contains("`ld.opt.pkg_warn` takes true or false"));
    }
}
