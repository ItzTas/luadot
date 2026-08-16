use crate::files::FileStatus;
use crate::output::Tone;

pub const DEFAULT_FILTER: &str = "warn";

pub const VERBOSE_FILTER: &str = "luadot=debug";

pub const TRACE_FILTER: &str = "luadot=trace";

pub const UNSET: &str = "(none)";

pub const UNDECLARED: &str = "(not declared)";

pub const DEFAULT_SHELL: &str = "/bin/sh";

pub const DEFAULT_REPO_DIR: &str = "repo";

pub const PREVIEW_LIMIT: usize = 10;

pub const YES_FLAGS: &str = "-y or --yes";

pub const TEMPLATE_SKELETON: &str = "return \"\"\n";

pub const STATUS_LABELS: [(FileStatus, &str, Tone); 4] = [
    (FileStatus::Synced, "synced", Tone::Good),
    (FileStatus::Missing, "missing", Tone::Warning),
    (FileStatus::Unlinked, "unlinked", Tone::Warning),
    (FileStatus::Differs, "differs", Tone::Bad),
];
