pub const APP_DIR: &str = "luadot";

pub const HOME_PREFIX: &str = "home";

pub const ROOT_PREFIX: &str = "root";

pub const DEFAULT_EDITOR: &str = "vi";

pub const DEFAULT_REPO_DIR: &str = "repo";

pub const SYSTEM_TEXT_MODE: u32 = 0o644;

pub const SPAN_UNITS: [(u64, &str); 4] = [
    (86_400, "day"),
    (3_600, "hour"),
    (60, "minute"),
    (1, "second"),
];

pub const SPAN_SUFFIXES: [(&str, u64); 5] = [
    ("s", 1),
    ("m", 60),
    ("h", 3_600),
    ("d", 86_400),
    ("w", 604_800),
];
