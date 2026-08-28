use glob::MatchOptions;

use super::diff::DiffState;

pub const CONFIG_FILE: &str = "config.lua";

pub const GIT_DIR: &str = ".git";

pub const MATCH: MatchOptions = MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
};

pub const PATH: &str = "path";

pub const SYSTEM: &str = "system";

pub const SOURCE: &str = "source";

pub const SIDE: &str = "side";

pub const STATE: &str = "state";

pub const CONTENT: &str = "content";

pub const MODE: &str = "mode";

pub const DRIFTED: &str = "drifted";

pub const TOTAL: &str = "total";

pub const TEMPLATES: &str = "templates";

pub const DEFAULT: &str = "default";

pub const NAME: &str = "name";

pub const DIFF_STATES: [(&str, DiffState); 4] = [
    ("missing", DiffState::Missing),
    ("differs", DiffState::Differs),
    ("mode", DiffState::Mode),
    ("other", DiffState::Other),
];

pub const LOCKED: &str = "the configuration is already being changed";

pub const BEFORE: &str = "before";

pub const AFTER: &str = "after";

pub const STARTER: &str = r#"-- The luadot configuration, read before every command.
-- `luadot doc ld` names every call, `luadot doc rules` describes one.

-- ld.opt.link("symbolic")

-- ld.rules({
--   { match = ".ssh/id_*", encrypt = true },
--   { match = ".config/mako/**", on_change = "makoctl reload" },
--   { match = ".cache/**", track = "never" },
-- })
"#;
