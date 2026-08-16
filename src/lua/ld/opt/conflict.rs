use mlua::{Function, Lua, Value};

use super::super::constants::CONFLICT_POLICIES;
use super::super::parse::lookup;
use super::super::surface::{self, Surface};
use super::constants::{CONFLICT, NAMESPACE};
use super::value::text;
use crate::lua::Config;

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, value: Value| set(lua, value))
}

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{CONFLICT}"), Surface::Config) {
        return Ok(());
    }

    let name = text(&value, CONFLICT)?;
    let policy = lookup(&CONFLICT_POLICIES, &name, "conflict policy")?;
    Config::building(lua)?.set_conflict(policy);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::files::ConflictPolicy;
    use crate::lua::from_source;

    #[test]
    fn sets_the_default_conflict_policy() {
        let config = from_source(r#"ld.opt.conflict("skip")"#).unwrap();

        assert_eq!(
            config.conflict_policy(Path::new(".bashrc")),
            ConflictPolicy::Skip
        );
    }

    #[test]
    fn rejects_an_unknown_conflict_policy() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.opt.conflict("explode")"#).unwrap_err()
        );

        assert!(err.contains("unknown conflict policy `explode`"));
        assert!(err.contains("overwrite, skip, error"));
    }

    #[test]
    fn rejects_a_value_that_is_not_a_string() {
        let err = format!("{:#}", from_source("ld.opt.conflict(true)").unwrap_err());

        assert!(err.contains("`ld.opt.conflict` takes a string"));
    }
}
