use super::types::Surface;

pub const CONFIG_NAME: &str = "config";

pub const BOOTSTRAP_NAME: &str = "bootstrap";

pub const SETUP_NAME: &str = "setup";

pub const TEMPLATE_NAME: &str = "template";

pub const STANDALONE_NAME: &str = "standalone";

pub const EXEC_NAME: &str = "exec";

pub const SURFACES: [Surface; 6] = [
    Surface::Config,
    Surface::Bootstrap,
    Surface::Setup,
    Surface::Template,
    Surface::Standalone,
    Surface::Exec,
];

pub const SETUP_LABEL: &str = "a setup script";

pub const EXEC_LABEL: &str = "luadot exec";

pub const STANDALONE_LABEL: &str = "a `.luadot` file";

pub const COMMAND_COST: &str = "runs before every command and will slow all of them down";

pub const TEMPLATE_COST: &str = "runs every time the template is resolved and will slow `alt` down";
