#[cfg(feature = "meta")]
use super::super::constants::CALL_METHOD;
#[cfg(feature = "meta")]
use super::super::signature::{Kind, Param, Signature};

pub const NAMESPACE: &str = "git";

pub const PROGRAM: &str = "git";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.git";

#[cfg(feature = "meta")]
pub const DOC: &str = "Runs git inside the managed repository: literal arguments, standard output returned, a non-zero status stops the script. A call before a repository is set stops instead of running git somewhere else. Slow: it belongs in `bootstrap.lua` or a setup script, and warns elsewhere.";

#[cfg(feature = "meta")]
pub const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[Param {
        name: "...",
        kind: Kind::Variadic(&Kind::String),
    }],
    returns: &[Kind::String],
    doc: "Runs git inside the repository and returns what it printed.",
};
