#[cfg(feature = "meta")]
use super::super::constants::CALL_METHOD;
#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind, Param, Signature};

pub const NAMESPACE: &str = "setup";

pub const ALL: &str = "all";

pub const LIST: &str = "list";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.setup";

#[cfg(feature = "meta")]
pub const DOC: &str = "The setup scripts of the repository, under `.config/luadot/setup/`: `<name>.lua`, `<name>.sh`, or a `<name>/` directory holding an `init.lua` or an `init.sh`. Running one is slow: it belongs in `bootstrap.lua`, and warns elsewhere.";

#[cfg(feature = "meta")]
pub const OPTIONS_TYPENAME: &str = "ld.SetupOptions";

#[cfg(feature = "meta")]
pub const OPTIONS_DOC: &str = "What `ld.setup.all` takes.";

#[cfg(feature = "meta")]
pub const ORDER: &str = "order";

#[cfg(feature = "meta")]
pub const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[Param {
        name: "name",
        kind: Kind::String,
    }],
    returns: &[],
    doc: "Runs one setup script by name.",
};

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 2] = [
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

#[cfg(feature = "meta")]
pub const OPTIONS_FIELDS: [Field; 1] = [Field {
    name: ORDER,
    kind: Kind::Optional(&Kind::List(&Kind::String)),
    doc: "The names that run first, in this order; the rest follow.",
}];
