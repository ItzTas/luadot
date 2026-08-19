use crate::files::FileStatus;
use crate::output::Tone;

pub const DEFAULT_FILTER: &str = "warn";

pub const VERBOSE_FILTER: &str = "luadot=debug";

pub const TRACE_FILTER: &str = "luadot=trace";

pub const UNSET: &str = "(none)";

pub const UNDECLARED: &str = "(not declared)";

pub const DEFAULT_SHELL: &str = "/bin/sh";

pub const PREVIEW_LIMIT: usize = 10;

pub const YES_FLAGS: &str = "-y or --yes";

pub const TEMPLATE_SKELETON: &str = "return \"\"\n";

pub const DIFF_PROGRAM: &str = "git";

pub const DIFF_ARGUMENTS: [&str; 3] = ["diff", "--no-index", "--no-prefix"];

pub const DIFF_SEPARATOR: &str = "--";

pub const DIFF_CUSTOM: &str = "ld.on.diff";

pub const STATUS_CUSTOM: &str = "ld.on.status";

pub const CUSTOM_ENTRY: &str = "entry";

pub const CUSTOM_RENDER: &str = "render";

pub const CUSTOM_SUMMARY: &str = "summary";

pub const MANAGED_FILES: &str = "managed";

pub const GENERATED_FILES: &str = "generated";

pub const STATUS_LABELS: [(FileStatus, &str, Tone); 5] = [
    (FileStatus::Synced, "synced", Tone::Good),
    (FileStatus::Missing, "missing", Tone::Warning),
    (FileStatus::Unlinked, "unlinked", Tone::Warning),
    (FileStatus::Differs, "differs", Tone::Bad),
    (FileStatus::Unreadable, "unreadable", Tone::Warning),
];
