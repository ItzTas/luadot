use crate::crypt::Backend;
use crate::files::{ConflictPolicy, LinkMode};

pub const API: &str = "ld";

pub const MATCH: &str = "match";

pub const REGEX: &str = "regex";

pub const LINK_MODES: [(&str, LinkMode); 3] = [
    ("hard", LinkMode::Hard),
    ("symbolic", LinkMode::Symbolic),
    ("copy", LinkMode::Copy),
];

pub const CONFLICT_POLICIES: [(&str, ConflictPolicy); 3] = [
    ("overwrite", ConflictPolicy::Overwrite),
    ("skip", ConflictPolicy::Skip),
    ("error", ConflictPolicy::Error),
];

pub const CRYPT_BACKENDS: [(&str, Backend); 2] = [("age", Backend::Age), ("gpg", Backend::Gpg)];
