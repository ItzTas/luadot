use mlua::{Lua, Value};

use super::super::value::path;
use super::constants::{BACKUP_DIR, NAMESPACE};
use crate::lua::Config;

pub fn set(lua: &Lua, value: Value) -> mlua::Result<()> {
    let dir = path(NAMESPACE, &value, BACKUP_DIR, "a directory")?;
    Config::building(lua, |config| config.set_backup_dir(dir))?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::lua::from_source;

    #[test]
    fn rejects_an_empty_directory() {
        let err = format!(
            "{:#}",
            from_source(r#"ld.opt.backup_dir("  ")"#).unwrap_err()
        );

        assert!(err.contains("`ld.opt.backup_dir` takes a directory"));
    }
}
