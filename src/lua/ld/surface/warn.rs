use mlua::Lua;

use super::super::constants::API;
use super::super::opt::{NAMESPACE, PKG_WARN};
use super::types::Surface;
use crate::lua::Config;
use crate::lua::bootstrap::constants::BOOTSTRAP_FILE;
use crate::output;

pub fn slow(lua: &Lua, call: &str) {
    let Some(surface) = Surface::current(lua) else {
        return;
    };
    let Some(message) = slow_message(surface, call) else {
        return;
    };
    if silenced(lua) {
        return;
    }

    output::warn(message);
}

pub fn slow_in(lua: &Lua, call: &str, home: Surface) {
    if Surface::current(lua) != Some(home) {
        return;
    }

    slow(lua, call);
}

pub fn inert(lua: &Lua, call: &str, home: Surface) -> bool {
    let Some(surface) = Surface::current(lua) else {
        return false;
    };
    if surface == home {
        return false;
    }
    if !silenced(lua) {
        output::warn(inert_message(surface, call, home));
    }

    true
}

fn slow_message(surface: Surface, call: &str) -> Option<String> {
    let cost = surface.cost()?;

    Some(format!(
        "`{API}.{call}` in {} {cost}; {BOOTSTRAP_FILE} is where it belongs ({})",
        surface.label(),
        silence()
    ))
}

fn inert_message(surface: Surface, call: &str, home: Surface) -> String {
    format!(
        "`{API}.{call}` in {} does nothing; {} is where it has an effect ({})",
        surface.label(),
        home.label(),
        silence()
    )
}

fn silence() -> String {
    format!("silence it with `{API}.{NAMESPACE}.{PKG_WARN}(false)`")
}

fn silenced(lua: &Lua) -> bool {
    let Ok(shared) = Config::shared(lua) else {
        return false;
    };
    let Ok(config) = shared.try_lock() else {
        return false;
    };

    !config.pkg_warn()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::lua::Shared;
    use crate::lua::runtime::runtime;

    fn running(surface: Surface) -> Lua {
        let lua = runtime().unwrap();
        surface.install(&lua);
        lua.set_app_data(Shared::new(Mutex::new(Config::default())));
        lua
    }

    #[test]
    fn the_configuration_surface_names_the_call_and_the_way_out() {
        let message = slow_message(Surface::Config, "pkg.install").unwrap();

        assert!(message.contains("`ld.pkg.install` in config.lua"));
        assert!(message.contains("runs before every command"));
        assert!(message.contains("bootstrap.lua is where it belongs"));
        assert!(message.contains("`ld.opt.pkg_warn(false)`"));
    }

    #[test]
    fn a_call_away_from_home_is_inert() {
        assert!(inert(
            &running(Surface::Bootstrap),
            "rules",
            Surface::Config
        ));
        assert!(inert(
            &running(Surface::Template),
            "crypt.lock",
            Surface::Config
        ));
    }

    #[test]
    fn silencing_the_warning_keeps_the_call_inert() {
        let lua = running(Surface::Bootstrap);
        Config::building(&lua, |config| config.set_pkg_warn(false)).unwrap();

        assert!(silenced(&lua));
        assert!(inert(&lua, "rules", Surface::Config));
    }
}
