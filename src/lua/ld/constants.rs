use crate::crypt::Backend;
use crate::files::{ConflictPolicy, LinkMode};
use crate::lua::Track;

pub const API: &str = "ld";

pub const MATCH: &str = "match";

pub const REGEX: &str = "regex";

pub const LINK: &str = "link";

pub const CONFLICT: &str = "conflict";

pub const ON_CHANGE: &str = "on_change";

pub const MODE: &str = "mode";

pub const SPECIAL_BITS: [(u32, &str); 2] = [(0o4000, "setuid"), (0o2000, "setgid")];

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

pub const TRACK_KINDS: [(&str, Track); 3] = [
    ("auto", Track::Auto),
    ("manual", Track::Manual),
    ("never", Track::Never),
];

pub const CRYPT_BACKENDS: [(&str, Backend); 2] = [("age", Backend::Age), ("gpg", Backend::Gpg)];

#[cfg(feature = "meta")]
pub const CALL_METHOD: &str = "__call";

#[cfg(feature = "meta")]
pub const STRING: &str = "string";

#[cfg(feature = "meta")]
pub const INTEGER: &str = "integer";

#[cfg(feature = "meta")]
pub const NUMBER: &str = "number";

#[cfg(feature = "meta")]
pub const BOOLEAN: &str = "boolean";

#[cfg(feature = "meta")]
pub const TABLE: &str = "table";

#[cfg(feature = "meta")]
pub const ANY: &str = "any";

#[cfg(feature = "meta")]
pub const NIL: &str = "nil";

#[cfg(feature = "meta")]
pub const FALSE: &str = "false";

#[cfg(feature = "meta")]
pub const LIGHT_USERDATA: &str = "lightuserdata";

#[cfg(feature = "meta")]
pub const STRING_INDEX: &str = "[string]";

#[cfg(feature = "meta")]
pub const INTEGER_INDEX: &str = "[integer]";

#[cfg(feature = "meta")]
pub const LINK_MODE_TYPENAME: &str = "ld.LinkMode";

#[cfg(feature = "meta")]
pub const CONFLICT_TYPENAME: &str = "ld.Conflict";

#[cfg(feature = "meta")]
pub const TRACK_TYPENAME: &str = "ld.Track";

#[cfg(feature = "meta")]
pub const BACKEND_TYPENAME: &str = "ld.Backend";
