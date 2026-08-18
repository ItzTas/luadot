use super::super::table::Builder;
use super::{diff, status};

pub const NAMESPACE: &str = "on";

pub const DIFF: &str = "diff";

pub const STATUS: &str = "status";

pub const FUNCTIONS: [(&str, Builder); 2] = [(DIFF, diff::function), (STATUS, status::function)];

pub const ARGS: &str = "args";

pub const ENTRY: &str = "entry";

pub const RENDER: &str = "render";

pub const SUMMARY: &str = "summary";

pub const TOOL: &str = "tool";

pub const DIFF_KEYS: [&str; 5] = [ARGS, ENTRY, RENDER, SUMMARY, TOOL];

pub const REPORT_KEYS: [&str; 3] = [ENTRY, RENDER, SUMMARY];
