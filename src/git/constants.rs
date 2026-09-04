pub const PROGRAM: &str = "git";

pub const GIT_DIR: &str = ".git";

pub const HEAD: &str = "HEAD";

pub const PUSH: &str = "push";

pub const MESSAGE: &str = "sync";

pub const RULES_IGNORE: &str = "ignore";

pub const RULES_ATTRIBUTES: &str = "attributes";

pub const INFO_DIR: &str = "info";

pub const INFO_EXCLUDE: &str = "exclude";

pub const INFO_ATTRIBUTES: &str = "attributes";

pub const INFO_START: &str = "# luadot";

pub const INFO_END: &str = "# /luadot";

pub const LFS_FILTERS: [&str; 4] = [
    "filter.lfs.clean=git-lfs clean -- %f",
    "filter.lfs.smudge=git-lfs smudge -- %f",
    "filter.lfs.process=git-lfs filter-process",
    "filter.lfs.required=true",
];
