use glob::MatchOptions;

use super::diff::DiffState;

pub const CONFIG_FILE: &str = "config.lua";

pub const GIT_DIR: &str = ".git";

pub const CLASS_QUESTION: &str = "define the class";

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

pub const DIFF_STATES: [(&str, DiffState); 4] = [
    ("missing", DiffState::Missing),
    ("differs", DiffState::Differs),
    ("mode", DiffState::Mode),
    ("other", DiffState::Other),
];

pub const MISSING: &str = "the configuration is not available";

pub const LOCKED: &str = "the configuration is already being changed";
