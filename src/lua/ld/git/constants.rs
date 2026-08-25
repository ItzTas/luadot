#[cfg(feature = "meta")]
use super::super::constants::CALL_METHOD;
#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind, Param, Signature};

pub const NAMESPACE: &str = "git";

pub const PROGRAM: &str = "git";

pub const CLONE: &str = "clone";

pub const AT: &str = "at";

pub const BRANCH: &str = "branch";

pub const DEPTH: &str = "depth";

pub const CLONE_KEYS: [&str; 2] = [BRANCH, DEPTH];

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.git";

#[cfg(feature = "meta")]
pub const DOC: &str = "Runs git inside the managed repository: literal arguments, standard output returned, a non-zero status stops the script. A call before a repository is set stops instead of running git somewhere else; `ld.git.clone` and `ld.git.at` reach other repositories. Slow: it belongs in `bootstrap.lua` or a setup script.";

#[cfg(feature = "meta")]
pub const CLONE_OPTIONS_TYPENAME: &str = "ld.CloneOptions";

#[cfg(feature = "meta")]
pub const CLONE_OPTIONS_DOC: &str = "What `ld.git.clone` takes besides the url and the directory.";

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

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 2] = [
    Signature {
        name: CLONE,
        params: &[
            Param {
                name: "url",
                kind: Kind::String,
            },
            Param {
                name: "dir",
                kind: Kind::String,
            },
            Param {
                name: "options",
                kind: Kind::Optional(&Kind::Named(CLONE_OPTIONS_TYPENAME)),
            },
        ],
        returns: &[],
        doc: "Clones a repository into that directory, which has to be empty or missing, without the `git` binary. `~` and a relative directory resolve against your home directory.",
    },
    Signature {
        name: AT,
        params: &[Param {
            name: "dir",
            kind: Kind::String,
        }],
        returns: &[Kind::Function(
            &[Param {
                name: "...",
                kind: Kind::Variadic(&Kind::String),
            }],
            &[Kind::String],
        )],
        doc: "A function running git inside that directory the way `ld.git` runs it inside the managed repository: `ld.git.at(dir)(\"fetch\", \"--tags\")`.",
    },
];

#[cfg(feature = "meta")]
pub const CLONE_FIELDS: [Field; 2] = [
    Field {
        name: BRANCH,
        kind: Kind::Optional(&Kind::String),
        doc: "The branch to check out. Defaults to the remote's `HEAD`.",
    },
    Field {
        name: DEPTH,
        kind: Kind::Optional(&Kind::Integer),
        doc: "How many commits of history to fetch, one or more; `1` is the commit the branch points at alone. Defaults to all of it.",
    },
];
