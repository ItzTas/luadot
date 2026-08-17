use mlua::{Lua, Value};

use super::super::surface::{self, Surface};
use super::super::value::path;
use super::constants::{BACKUP_DIR, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    if surface::inert(lua, &format!("{NAMESPACE}.{BACKUP_DIR}"), Surface::Config) {
        return Ok(());
    }

    let dir = path(NAMESPACE, &value, BACKUP_DIR, "a directory")?;
    Config::building(lua)?.set_backup_dir(dir);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::lua::from_source;

    #[test]
    fn defaults_to_the_directory_luadot_owns() {
        let config = from_source("local unused = 1").unwrap();

        assert_eq!(config.backup_dir(), None);
    }

    #[test]
    fn takes_the_directory_as_it_is_written() {
        let config = from_source(r#"ld.opt.backup_dir("~/dots/backups")"#).unwrap();

        assert_eq!(config.backup_dir(), Some(Path::new("~/dots/backups")));
    }

    #[test]
    fn the_last_directory_wins() {
        let config = from_source(
            r#"
            ld.opt.backup_dir("~/first")
            ld.opt.backup_dir("/data/second")
            "#,
        )
        .unwrap();

        assert_eq!(config.backup_dir(), Some(Path::new("/data/second")));
    }

    #[test]
    fn rejects_a_value_that_is_not_a_string() {
        let err = format!("{:#}", from_source("ld.opt.backup_dir(true)").unwrap_err());

        assert!(err.contains("`ld.opt.backup_dir` takes a string"));
    }

    #[test]
    fn rejects_an_empty_directory() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.opt.backup_dir("  ")"#).unwrap_err()
        );

        assert!(err.contains("`ld.opt.backup_dir` takes a directory"));
    }
}
