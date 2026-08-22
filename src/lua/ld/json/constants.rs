#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind, Param, Signature};

pub const NAMESPACE: &str = "json";

pub const ENCODE: &str = "encode";

pub const DECODE: &str = "decode";

pub const NULL: &str = "null";

pub const DEPTH: usize = 64;

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.json";

#[cfg(feature = "meta")]
pub const DOC: &str = "JSON in and out. A table is a list or a table of names, never both, and `null` has a value of its own, since `nil` cannot sit in a table.";

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 2] = [
    Signature {
        name: ENCODE,
        params: &[Param {
            name: "value",
            kind: Kind::Any,
        }],
        returns: &[Kind::String],
        doc: "That value as JSON, indented, with sorted keys; `ld.alt.json` is the same call.",
    },
    Signature {
        name: DECODE,
        params: &[Param {
            name: "text",
            kind: Kind::String,
        }],
        returns: &[Kind::Any],
        doc: "The value the text holds: an object or a list as a table, a whole number as an integer, `null` as `ld.json.null`. A text that is not JSON stops the script.",
    },
];

#[cfg(feature = "meta")]
pub const FIELDS: [Field; 1] = [Field {
    name: NULL,
    kind: Kind::LightUserData,
    doc: "What a JSON `null` decodes to, and what encodes back as one. `nil` encodes as `null` too, but cannot sit in a table.",
}];
