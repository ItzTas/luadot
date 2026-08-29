use super::super::constants::{CONFLICT, LINK, MATCH, MODE, ON_CHANGE, REGEX};

pub const RULES: &str = "rules";

pub const TRACK: &str = "track";

pub const OWNER: &str = "owner";

pub const ENCRYPT: &str = "encrypt";

pub const LFS: &str = "lfs";

pub const AUTOCOMMIT: &str = "autocommit";

pub const AUTOPUSH: &str = "autopush";

pub const WHOLE: &str = "whole";

pub const RULE_KEYS: [&str; 13] = [
    MATCH, REGEX, LINK, CONFLICT, ON_CHANGE, TRACK, MODE, OWNER, ENCRYPT, LFS, AUTOCOMMIT,
    AUTOPUSH, WHOLE,
];

pub const TASK: &str = "task";

pub const ABOUT: &str = "about";

pub const RUN: &str = "run";

pub const TASK_KEYS: [&str; 2] = [ABOUT, RUN];

pub const BUILTINS: [&str; 28] = [
    "init",
    "clone",
    "add",
    "take",
    "rm",
    "mv",
    "status",
    "diff",
    "apply",
    "tmpl",
    "restore",
    "edit",
    "rekey",
    "exec",
    "config",
    "class",
    "bootstrap",
    "setup",
    "task",
    "cd",
    "sync",
    "git",
    "push",
    "doc",
    "meta",
    "completions",
    "man",
    "help",
];

pub const SURFACE: &str = "surface";
