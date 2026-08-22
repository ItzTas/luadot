use mlua::Lua;

use super::constants::{
    BOOTSTRAP_NAME, COMMAND_COST, CONFIG_NAME, EXEC_LABEL, EXEC_NAME, SETUP_LABEL, SETUP_NAME,
    STANDALONE_LABEL, STANDALONE_NAME, TEMPLATE_COST, TEMPLATE_NAME,
};
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

    pub fn name(self) -> &'static str {
        match self {
            Self::Config => CONFIG_NAME,
            Self::Bootstrap => BOOTSTRAP_NAME,
            Self::Setup => SETUP_NAME,
            Self::Template => TEMPLATE_NAME,
            Self::Standalone => STANDALONE_NAME,
            Self::Exec => EXEC_NAME,
        }
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
    use super::super::constants::SURFACES;
    use super::*;

    #[test]
    fn every_surface_answers_to_a_name_of_its_own() {
        assert_eq!(
            SURFACES.map(Surface::name),
            [
                "config",
                "bootstrap",
                "setup",
                "template",
                "standalone",
                "exec"
            ]
        );
    }
}
