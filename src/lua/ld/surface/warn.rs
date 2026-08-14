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
    lua.app_data_ref::<Config>()
        .is_some_and(|config| !config.pkg_warn())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    fn running(surface: Surface) -> Lua {
        let lua = runtime().unwrap();
        surface.install(&lua);
        lua.set_app_data(Config::default());
        lua
    }

    #[test]
    fn the_configuration_surface_names_the_call_and_the_way_out() {
        let message = slow_message(Surface::Config, "pkg.install").unwrap();

        assert!(message.contains("`ld.pkg.install` in ld.lua"));
        assert!(message.contains("runs before every command"));
        assert!(message.contains("bootstrap.lua is where it belongs"));
        assert!(message.contains("`ld.opt.pkg_warn(false)`"));
    }

    #[test]
    fn a_template_is_warned_about_the_cost_it_pays_on_every_resolution() {
        let message = slow_message(Surface::Template, "cmd").unwrap();

        assert!(message.contains("`ld.cmd` in luadot.lua"));
        assert!(message.contains("runs every time the template is resolved"));
    }

    #[test]
    fn the_surfaces_running_once_carry_no_slow_message() {
        assert!(slow_message(Surface::Bootstrap, "pkg.install").is_none());
        assert!(slow_message(Surface::Setup, "setup.all").is_none());
    }

    #[test]
    fn an_inert_call_names_the_surface_it_belongs_to() {
        let message = inert_message(Surface::Setup, "rules", Surface::Config);

        assert!(message.contains("`ld.rules` in a setup script does nothing"));
        assert!(message.contains("ld.lua is where it has an effect"));
        assert!(message.contains("`ld.opt.pkg_warn(false)`"));
    }

    #[test]
    fn a_call_at_home_is_never_inert() {
        assert!(!inert(&running(Surface::Config), "rules", Surface::Config));
        assert!(!inert(
            &running(Surface::Template),
            "alt.out",
            Surface::Template
        ));
    }

    #[test]
    fn a_call_away_from_home_is_inert() {
        assert!(inert(
            &running(Surface::Bootstrap),
            "rules",
            Surface::Config
        ));
        assert!(inert(
            &running(Surface::Config),
            "alt.out",
            Surface::Template
        ));
    }

    #[test]
    fn silencing_the_warning_keeps_the_call_inert() {
        let lua = running(Surface::Bootstrap);
        lua.app_data_mut::<Config>().unwrap().set_pkg_warn(false);

        assert!(silenced(&lua));
        assert!(inert(&lua, "rules", Surface::Config));
    }

    #[test]
    fn a_runtime_without_a_surface_stays_out_of_the_way() {
        let lua = runtime().unwrap();

        assert!(!inert(&lua, "rules", Surface::Config));
    }
}
