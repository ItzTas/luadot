#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind};

pub const NAMESPACE: &str = "argv";

pub const NAME: &str = "name";

pub const ARGS: &str = "args";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.argv";

#[cfg(feature = "meta")]
pub const DOC: &str =
    "The invocation: `luadot apply .config/nvim` gives `\"apply\"` and `{ \".config/nvim\" }`.";

#[cfg(feature = "meta")]
pub const FIELDS: [Field; 2] = [
    Field {
        name: NAME,
        kind: Kind::String,
        doc: "The command as typed.",
    },
    Field {
        name: ARGS,
        kind: Kind::List(&Kind::String),
        doc: "Everything after the command.",
    },
];
