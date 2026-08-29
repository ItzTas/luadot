use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, Kind, Param, Signature};
use super::constants::PAGE;

const NAMESPACE_TYPENAME: &str = "ld.doc";

const DOC: &str = "The pages `luadot doc` answers from besides luadot's own.";

const SIGNATURES: [Signature; 1] = [Signature {
    name: PAGE,
    params: &[Param {
        name: "path",
        kind: Kind::String,
    }],
    returns: &[],
    doc: "Registers a markdown page for `luadot doc`. Every table row whose first cell is a namespaced call in backticks, like `lazyld.sync(names)`, is answered the way the calls of the interface are: the second cell is what it takes, the third what it does. `~` and a relative path resolve against your home directory. Only `config.lua` registers; elsewhere the call does nothing and says so.",
}];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker.namespace(NAMESPACE_TYPENAME, DOC, |record| {
        record.functions(&SIGNATURES)
    })
}
