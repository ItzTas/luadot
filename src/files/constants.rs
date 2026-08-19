pub const COMMAND: &str = "files";

pub const GIT_DIR: &str = ".git";

pub const TEMPLATE_SUFFIX: &str = ".luadot";

pub const MODE_BITS: u32 = 0o7777;

pub const SUDO: &str = "sudo";

pub const STDIN_PATH: &str = "/dev/stdin";

pub const MIRROR_PREFIX: &str = "luadot-diff";

pub const MIRROR_MODE: u32 = 0o700;

pub const MIRROR_TREE: &str = "tree";

pub const MIRROR_GIT: &str = "git";

pub const MIRROR_INIT: [&str; 4] = ["init", "--quiet", "--initial-branch", "luadot"];

pub const MIRROR_ADD: [&str; 3] = ["add", "--force", "--all"];

pub const REPOSITORY_SIDE: &str = "repository";

pub const GENERATED_SIDE: &str = "generated";

pub const SYSTEM_SIDE: &str = "system";
