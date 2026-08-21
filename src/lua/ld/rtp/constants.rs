#[cfg(feature = "meta")]
use super::super::signature::{Kind, Param, Signature};

pub const NAMESPACE: &str = "rtp";

pub const ADD: &str = "add";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.rtp";

#[cfg(feature = "meta")]
pub const DOC: &str = "The directories `require` searches besides the configuration's own `lua/`: what a plugin manager registers, carried to every script the command runs.";

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 1] = [Signature {
    name: ADD,
    params: &[Param {
        name: "dir",
        kind: Kind::String,
    }],
    returns: &[],
    doc: "Puts `<dir>/lua/` on the module path of this script and of every script the command runs after it, behind the configuration's own `lua/` and in the order registered. `~` and a relative path resolve against your home directory; a directory added twice is kept once.",
}];
