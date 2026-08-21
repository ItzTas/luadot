use mlua::Lua;

use super::constants::{COMMAND_COST, EXEC_LABEL, SETUP_LABEL, STANDALONE_LABEL, TEMPLATE_COST};
use crate::lua::bootstrap::constants::BOOTSTRAP_FILE;
use crate::lua::config::constants::CONFIG_FILE;
use crate::lua::template::constants::TEMPLATE_FILE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    Config,
    Bootstrap,
    Setup,
    Template,
    Standalone,
    Exec,
}

impl Surface {
    pub fn install(self, lua: &Lua) {
        lua.set_app_data(self);
    }

    pub fn current(lua: &Lua) -> Option<Self> {
        lua.app_data_ref::<Self>().map(|surface| *surface)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Config => CONFIG_FILE,
            Self::Bootstrap => BOOTSTRAP_FILE,
            Self::Setup => SETUP_LABEL,
            Self::Template => TEMPLATE_FILE,
            Self::Standalone => STANDALONE_LABEL,
            Self::Exec => EXEC_LABEL,
        }
    }

    pub fn cost(self) -> Option<&'static str> {
        match self {
            Self::Config => Some(COMMAND_COST),
            Self::Template | Self::Standalone => Some(TEMPLATE_COST),
            Self::Bootstrap | Self::Setup | Self::Exec => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    #[test]
    fn a_bootstrap_and_a_setup_script_are_interchangeable() {
        assert_eq!(Surface::Bootstrap.cost(), Surface::Setup.cost());
    }

    #[test]
    fn the_installed_surface_is_the_current_one() {
        let lua = runtime().unwrap();

        assert_eq!(Surface::current(&lua), None);

        Surface::Template.install(&lua);

        assert_eq!(Surface::current(&lua), Some(Surface::Template));
    }
}
