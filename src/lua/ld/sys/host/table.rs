use std::env;

use mlua::{Lua, Table};

use crate::utils;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let host = lua.create_table()?;
    host.set("name", utils::host_name())?;
    host.set("os", env::consts::OS)?;
    host.set("arch", env::consts::ARCH)?;

    Ok(host)
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
}
