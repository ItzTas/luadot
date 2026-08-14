use mlua::{Lua, Table};

use super::constants::HAS_BATTERY;
use super::{gpu, has_battery, host, ram};

type Namespace = fn(&Lua) -> mlua::Result<Table>;

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    let namespaces: [(&str, Namespace); 2] =
        [(gpu::NAMESPACE, gpu::table), (host::NAMESPACE, host::table)];

    let sys = lua.create_table()?;
    for (name, namespace) in namespaces {
        sys.set(name, namespace(lua)?)?;
    }
    sys.set(ram::NAMESPACE, ram::total())?;
    sys.set(HAS_BATTERY, has_battery::function(lua)?)?;

    Ok(sys)
}

#[cfg(test)]
mod tests {
    use mlua::Function;

    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn the_machine_answers_through_every_name() {
        let lua = runtime().unwrap();

        let sys = table(&lua).unwrap();

        for name in ["gpu", "host"] {
            assert!(sys.get::<Table>(name).is_ok(), "{name} is missing");
        }
        assert!(sys.get::<u64>("ram").is_ok());
        assert!(
            sys.get::<Function>(HAS_BATTERY)
                .unwrap()
                .call::<bool>(())
                .is_ok()
        );
    }
}
