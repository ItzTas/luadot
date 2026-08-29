use tealr::TypeWalker;

use super::super::constants::CALL_METHOD;
use super::super::signature::{Collect, Describe, Field, Kind, Param, Signature, record};
use super::constants::{ALL, LIST};

const NAMESPACE_TYPENAME: &str = "ld.setup";

const DOC: &str = "The setup scripts of the repository, under `.config/luadot/setup/`: `<name>.lua`, `<name>.sh`, or a `<name>/` directory holding an `init.lua` or an `init.sh`. Running one is slow: it belongs in `bootstrap.lua`.";

const OPTIONS_TYPENAME: &str = "ld.SetupOptions";

const OPTIONS_DOC: &str = "What `ld.setup.all` takes.";

const ORDER: &str = "order";

const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[Param {
        name: "name",
        kind: Kind::String,
    }],
    returns: &[],
    doc: "Runs one setup script by name.",
};

const SIGNATURES: [Signature; 2] = [
    Signature {
        name: LIST,
        params: &[],
        returns: &[Kind::List(&Kind::String)],
        doc: "The names of the available setup scripts, directories included.",
    },
    Signature {
        name: ALL,
        params: &[Param {
            name: "options",
            kind: Kind::Optional(&Kind::Named(OPTIONS_TYPENAME)),
        }],
        returns: &[],
        doc: "Runs every setup script, the ones `order` names first.",
    },
];

const OPTIONS_FIELDS: [Field; 1] = [Field {
    name: ORDER,
    kind: Kind::Optional(&Kind::List(&Kind::String)),
    doc: "The names that run first, in this order; the rest follow.",
}];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES).call(&CALL)
        })
        .record(record(OPTIONS_TYPENAME, OPTIONS_DOC).fields(&OPTIONS_FIELDS))
}
