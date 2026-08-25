#[cfg(feature = "meta")]
use super::super::constants::{
    CONFLICT, CONFLICT_TYPENAME, LINK, LINK_MODE_TYPENAME, MODE, ON_CHANGE,
};
#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind, Param, Signature};

pub const NAMESPACE: &str = "alt";

pub const OUT: &str = "out";

pub const FILE: &str = "file";

pub const RENDER: &str = "render";

pub const EXPAND: &str = "expand";

pub const READ: &str = "read";

pub const EXISTS: &str = "exists";

pub const GLOB: &str = "glob";

pub const JSON: &str = "json";

pub const DEST: &str = "dest";

pub const CONTENT: &str = "content";

pub const DEST_ALONE: &str =
    "needs a `dest`: only a template knows the file it stands for by itself";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.alt";

#[cfg(feature = "meta")]
pub const DOC: &str = "The files of a template, resolved against the directory the running script lives in: the template directory inside a template, `ld.path.dir` anywhere else. A relative name starts there; an absolute one, or one climbing out with `..`, reaches anywhere.";

#[cfg(feature = "meta")]
pub const OUTPUT_TYPENAME: &str = "ld.Output";

#[cfg(feature = "meta")]
pub const OUTPUT_DOC: &str =
    "A file a template produces, as `ld.alt.out` takes it or `luadot.lua` returns it.";

#[cfg(feature = "meta")]
pub const FILE_TYPENAME: &str = "ld.File";

#[cfg(feature = "meta")]
pub const FILE_DOC: &str = "A file of the template as `ld.alt.file` hands it over, linked to its destination the way a managed file is.";

#[cfg(feature = "meta")]
const NAME_PARAM: Param = Param {
    name: "name",
    kind: Kind::String,
};

#[cfg(feature = "meta")]
const VARS_PARAM: Param = Param {
    name: "vars",
    kind: Kind::Optional(&Kind::Table),
};

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 8] = [
    Signature {
        name: OUT,
        params: &[Param {
            name: "file",
            kind: Kind::Or(&[
                Kind::Named(OUTPUT_TYPENAME),
                Kind::String,
                Kind::Named(FILE_TYPENAME),
            ]),
        }],
        returns: &[],
        doc: "Declares a file the template produces; repeated calls accumulate. Outside a template it writes the file where `dest` says, straight away.",
    },
    Signature {
        name: FILE,
        params: &[NAME_PARAM],
        returns: &[Kind::Named(FILE_TYPENAME)],
        doc: "A real file, linked to the destination like a managed one.",
    },
    Signature {
        name: RENDER,
        params: &[NAME_PARAM, VARS_PARAM],
        returns: &[Kind::String],
        doc: "Runs that Lua file with `vars` in scope and returns the string it returns.",
    },
    Signature {
        name: EXPAND,
        params: &[NAME_PARAM, VARS_PARAM],
        returns: &[Kind::String],
        doc: "Renders that embedded template, text as it stands and Lua between `<%` and `%>`, with `vars` in scope, and returns the string it emits.",
    },
    Signature {
        name: READ,
        params: &[NAME_PARAM],
        returns: &[Kind::String],
        doc: "What that file holds, as a string, never run.",
    },
    Signature {
        name: EXISTS,
        params: &[NAME_PARAM],
        returns: &[Kind::Boolean],
        doc: "Whether that file is there.",
    },
    Signature {
        name: GLOB,
        params: &[Param {
            name: "pattern",
            kind: Kind::String,
        }],
        returns: &[Kind::List(&Kind::String)],
        doc: "The names of the files it matches, sorted, named the way `ld.alt.read` takes them; directories are never listed.",
    },
    Signature {
        name: JSON,
        params: &[Param {
            name: "value",
            kind: Kind::Any,
        }],
        returns: &[Kind::String],
        doc: "That value as JSON, indented, with sorted keys. A table is a list or a table of names, never both. The same call as `ld.json.encode`.",
    },
];

#[cfg(feature = "meta")]
pub const OUTPUT_FIELDS: [Field; 6] = [
    Field {
        name: CONTENT,
        kind: Kind::Or(&[Kind::String, Kind::Named(FILE_TYPENAME)]),
        doc: "What lands on the system: a string is written, a file is linked. Required.",
    },
    Field {
        name: DEST,
        kind: Kind::Optional(&Kind::String),
        doc: "Where it lands; `~/` and a relative path both start at your home directory. Defaults to the mirrored path.",
    },
    Field {
        name: LINK,
        kind: Kind::Optional(&Kind::Named(LINK_MODE_TYPENAME)),
        doc: "How an `ld.alt.file` is placed. Defaults to the configured mode.",
    },
    Field {
        name: CONFLICT,
        kind: Kind::Optional(&Kind::Named(CONFLICT_TYPENAME)),
        doc: "Answer when the destination already holds something else. Defaults to the configured policy.",
    },
    Field {
        name: MODE,
        kind: Kind::Optional(&Kind::String),
        doc: "Three or four octal digits, the permissions of the generated file, `\"600\"` for one holding a secret. Only for generated content: an `ld.alt.file` keeps its own mode.",
    },
    Field {
        name: ON_CHANGE,
        kind: Kind::Optional(&Kind::String),
        doc: "A command line run through `sh -c` after the file is created or replaced, and only then. Wins over an `on_change` rule matching the same path.",
    },
];
