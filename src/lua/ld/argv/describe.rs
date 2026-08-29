use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, Field, Kind};
use super::constants::{ARGS, NAME};

const NAMESPACE_TYPENAME: &str = "ld.argv";

const DOC: &str =
    "The invocation: `luadot apply .config/nvim` gives `\"apply\"` and `{ \".config/nvim\" }`.";

const FIELDS: [Field; 2] = [
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

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker.namespace(NAMESPACE_TYPENAME, DOC, |record| record.fields(&FIELDS))
}
