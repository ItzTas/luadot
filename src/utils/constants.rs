pub const APP_DIR: &str = "luadot";

pub const DEFAULT_CONFIG_DIR: &str = ".config";

pub const DEFAULT_DATA_DIR: &str = ".local/share";

pub const DEFAULT_EDITOR: &str = "vi";

pub const DEFAULT_REPO_DIR: &str = "repo";

pub const SYSTEM_TEXT_MODE: u32 = 0o644;

pub const DEFINITIONS_WROTE: &str = "wrote";

pub const DEFINITIONS_MERGED: &str = "merged";

pub const DEFINITIONS_KEPT: &str = "could not be parsed and was left alone; add this to it:";

pub const HOSTNAME_FILES: [&str; 2] = ["/proc/sys/kernel/hostname", "/etc/hostname"];

pub const HOSTNAME_VAR: &str = "HOSTNAME";

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
