pub const ZSH_DISPATCH: &str = "if [ \"$funcstack[1]\"";

pub const BASH_GIT_COMPLETION: &str = include_str!("../../completions/luadot.bash");

pub const ZSH_GIT_COMPLETION: &str = include_str!("../../completions/luadot.zsh");

pub const FISH_GIT_COMPLETION: &str = include_str!("../../completions/luadot.fish");
