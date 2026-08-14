use std::env;
use std::path::Path;

use mlua::{Lua, Table};

use super::constants::{HOSTNAME_FILES, HOSTNAME_VAR};

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let host = lua.create_table()?;
    host.set("name", name())?;
    host.set("os", env::consts::OS)?;
    host.set("arch", env::consts::ARCH)?;

    Ok(host)
}

fn name() -> String {
    HOSTNAME_FILES
        .iter()
        .find_map(|path| read(Path::new(path)))
        .or_else(|| env::var(HOSTNAME_VAR).ok().and_then(trimmed))
        .unwrap_or_default()
}

fn read(path: &Path) -> Option<String> {
    trimmed(std::fs::read_to_string(path).ok()?)
}

fn trimmed(value: String) -> Option<String> {
    let value = value.trim();

    (!value.is_empty()).then(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn exposes_the_machine_as_strings() {
        let lua = runtime().unwrap();

        let host = table(&lua).unwrap();

        assert!(host.get::<String>("name").is_ok());
        assert_eq!(host.get::<String>("os").unwrap(), env::consts::OS);
        assert_eq!(host.get::<String>("arch").unwrap(), env::consts::ARCH);
    }

    #[test]
    fn trimmed_drops_the_newline_and_the_empty_value() {
        assert_eq!(
            trimmed("thinkpad\n".to_string()),
            Some("thinkpad".to_string())
        );
        assert_eq!(trimmed("  \n".to_string()), None);
    }

    #[test]
    fn read_ignores_a_missing_file() {
        assert_eq!(read(Path::new("/nonexistent/hostname")), None);
    }
}
