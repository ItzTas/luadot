use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, Kind, Param, Signature};
use super::constants::INSTALL;

const NAMESPACE_TYPENAME: &str = "ld.pkg";

const DOC: &str = "The system package manager: pacman, apt-get or dnf, whichever is on the `PATH`, through `sudo` when it is there.";

const SIGNATURES: [Signature; 1] = [Signature {
    name: INSTALL,
    params: &[Param {
        name: "packages",
        kind: Kind::Or(&[Kind::String, Kind::List(&Kind::String)]),
    }],
    returns: &[],
    doc: "Installs packages through the system package manager. Slow: it belongs in `bootstrap.lua` or a setup script.",
}];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker.namespace(NAMESPACE_TYPENAME, DOC, |record| {
        record.functions(&SIGNATURES)
    })
}
