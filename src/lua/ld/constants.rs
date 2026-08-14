use crate::files::{ConflictPolicy, LinkMode};

pub const API: &str = "ld";

pub const LINK_MODES: [(&str, LinkMode); 2] =
    [("hard", LinkMode::Hard), ("symbolic", LinkMode::Symbolic)];

pub const CONFLICT_POLICIES: [(&str, ConflictPolicy); 3] = [
    ("overwrite", ConflictPolicy::Overwrite),
    ("skip", ConflictPolicy::Skip),
    ("error", ConflictPolicy::Error),
];
