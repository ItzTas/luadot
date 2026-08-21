pub const FETCH_TASK: &str = "fetch";

pub const CHECKOUT_TASK: &str = "checkout";

pub const PROGRAM: &str = "git";

pub const GIT_DIR: &str = ".git";

pub const HEAD: &str = "HEAD";

pub const SEPARATOR: &str = "--";

pub const ADD: [&str; 1] = ["add"];

pub const ADD_ALL: [&str; 2] = ["add", "-A"];

pub const REMOVE: [&str; 5] = ["rm", "-r", "--cached", "--ignore-unmatch", "--quiet"];

pub const STAGED: [&str; 3] = ["diff", "--cached", "--quiet"];

pub const COMMITTED: [&str; 3] = ["rev-parse", "--verify", "HEAD"];

pub const COMMIT: &str = "commit";

pub const MESSAGE_FLAG: &str = "-m";

pub const PUSH: &str = "push";

pub const UPSTREAM: [&str; 3] = ["rev-parse", "--abbrev-ref", "@{upstream}"];

pub const SET_UPSTREAM: [&str; 3] = ["--set-upstream", "origin", "HEAD"];

pub const MESSAGE: &str = "sync";

pub const MESSAGE_FROM: &str = "sync from";

pub const LFS_PROGRAM: &str = "git-lfs";

pub const LFS_VERSION: &str = "version";

pub const LFS_INSTALL: [&str; 3] = ["lfs", "install", "--local"];

pub const ATTRIBUTES_FILE: &str = ".gitattributes";

pub const MARKER_START: &str = "# luadot:lfs";

pub const MARKER_END: &str = "# /luadot:lfs";

pub const TRACKED: &str = "filter=lfs diff=lfs merge=lfs -text";

pub const UNTRACKED: &str = "-filter -diff -merge text";

pub const LFS_FILTERS: [&str; 4] = [
    "filter.lfs.clean=git-lfs clean -- %f",
    "filter.lfs.smudge=git-lfs smudge -- %f",
    "filter.lfs.process=git-lfs filter-process",
    "filter.lfs.required=true",
];

pub const INTERRUPT_GRACE: usize = 0;
