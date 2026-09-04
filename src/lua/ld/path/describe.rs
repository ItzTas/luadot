use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, Field, Kind};
use super::constants::{CONFIG, DATA, DIR, HOME, REPO};

const NAMESPACE_TYPENAME: &str = "ld.path";

const DOC: &str = "The directories of the run.";

const FIELDS: [Field; 5] = [
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
        name: DATA,
        kind: Kind::String,
        doc: "The data directory, `~/.local/share/luadot`, where the state, the backups and the default repository live. luadot owns no subdirectory a plugin manager might pick there.",
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

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker.namespace(NAMESPACE_TYPENAME, DOC, |record| record.fields(&FIELDS))
}
