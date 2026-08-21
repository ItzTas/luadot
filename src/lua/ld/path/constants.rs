#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind};

pub const NAMESPACE: &str = "path";

pub const HOME: &str = "home";

pub const CONFIG: &str = "config";

pub const REPO: &str = "repo";

pub const DIR: &str = "dir";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.path";

#[cfg(feature = "meta")]
pub const DOC: &str = "The directories of the run.";

#[cfg(feature = "meta")]
pub const FIELDS: [Field; 4] = [
    Field {
        name: HOME,
        kind: Kind::String,
        doc: "Your home directory.",
    },
    Field {
        name: CONFIG,
        kind: Kind::String,
        doc: "The configuration directory, `~/.config/luadot`.",
    },
    Field {
        name: REPO,
        kind: Kind::Optional(&Kind::String),
        doc: "The managed repository, once one is set. Inside `config.lua` it is the one known before the file ran, so it does not answer for an `ld.opt.repo_dir` set in that same file.",
    },
    Field {
        name: DIR,
        kind: Kind::Optional(&Kind::String),
        doc: "The directory of the script that is running, the template directory inside a template.",
    },
];
