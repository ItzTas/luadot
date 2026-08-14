use glob::MatchOptions;

pub const CONFIG_FILE: &str = "ld.lua";

pub const GIT_DIR: &str = ".git";

pub const CLASS_QUESTION: &str = "define the class";

pub const MATCH: MatchOptions = MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
};
