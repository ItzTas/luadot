pub const FETCH_TASK: &str = "fetch";

pub const CHECKOUT_TASK: &str = "checkout";

pub const PROGRAM: &str = "git";

pub const GIT_DIR: &str = ".git";

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
