use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, Field, Kind, Param, Signature};
use super::constants::{DECODE, ENCODE, NULL};

const NAMESPACE_TYPENAME: &str = "ld.json";

const DOC: &str = "JSON in and out. A table is a list or a table of names, never both, and `null` has a value of its own, since `nil` cannot sit in a table.";

const SIGNATURES: [Signature; 2] = [
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

const FIELDS: [Field; 1] = [Field {
    name: NULL,
    kind: Kind::LightUserData,
    doc: "What a JSON `null` decodes to, and what encodes back as one. `nil` encodes as `null` too, but cannot sit in a table.",
}];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker.namespace(NAMESPACE_TYPENAME, DOC, |record| {
        record.fields(&FIELDS).functions(&SIGNATURES)
    })
}
