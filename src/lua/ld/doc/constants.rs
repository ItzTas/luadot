#[cfg(feature = "meta")]
use super::super::signature::{Kind, Param, Signature};

pub const NAMESPACE: &str = "doc";

pub const PAGE: &str = "page";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.doc";

#[cfg(feature = "meta")]
pub const DOC: &str = "The pages `luadot doc` answers from besides luadot's own.";

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 1] = [Signature {
    name: PAGE,
    params: &[Param {
        name: "path",
        kind: Kind::String,
    }],
    returns: &[],
    doc: "Registers a markdown page for `luadot doc`. Every table row whose first cell is a namespaced call in backticks, like `lazyld.sync(names)`, is answered the way the calls of the interface are: the second cell is what it takes, the third what it does. `~` and a relative path resolve against your home directory. Only `config.lua` registers; elsewhere the call does nothing and says so.",
}];
