use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, Kind, Param, Signature};
use super::constants::ADD;

const NAMESPACE_TYPENAME: &str = "ld.rtp";

const DOC: &str = "The directories `require` searches besides the configuration's own `lua/`: what a plugin manager registers, carried to every script the command runs.";

const SIGNATURES: [Signature; 1] = [Signature {
    name: ADD,
    params: &[Param {
        name: "dir",
        kind: Kind::String,
    }],
    returns: &[],
    doc: "Puts `<dir>/lua/` on the module path of this script and of every script the command runs after it, behind the configuration's own `lua/` and in the order registered. `~` and a relative path resolve against your home directory; a directory added twice is kept once.",
}];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker.namespace(NAMESPACE_TYPENAME, DOC, |record| {
        record.functions(&SIGNATURES)
    })
}
