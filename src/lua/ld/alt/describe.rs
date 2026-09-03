use tealr::TypeWalker;

use super::super::constants::{
    CONFLICT, CONFLICT_TYPENAME, LINK, LINK_MODE_TYPENAME, MODE, ON_CHANGE,
};
use super::super::signature::{Collect, Describe, Field, Kind, Param, Signature, record};
use super::constants::{
    CONCAT, CONTENT, DEST, EXISTS, EXPAND, FILE, GLOB, JSON, OUT, READ, RENDER, WHEN,
};

const NAMESPACE_TYPENAME: &str = "ld.alt";

const DOC: &str = "The files of a template, resolved against the directory the running script lives in: the template directory inside a template, `ld.path.dir` anywhere else. A relative name starts there; an absolute one, or one climbing out with `..`, reaches anywhere.";

const OUTPUT_TYPENAME: &str = "ld.Output";

const OUTPUT_DOC: &str =
    "A file a template produces, as `ld.alt.out` takes it or `luadot.lua` returns it.";

const FILE_TYPENAME: &str = "ld.File";

const FILE_DOC: &str = "A file of the template as `ld.alt.file` hands it over, linked to its destination the way a managed file is.";

const SECTION_TYPENAME: &str = "ld.Section";

const SECTION_DOC: &str =
    "One fragment of the file `ld.alt.concat` builds, and the condition it lands under.";

const NAME_PARAM: Param = Param {
    name: "name",
    kind: Kind::String,
};

const VARS_PARAM: Param = Param {
    name: "vars",
    kind: Kind::Optional(&Kind::Table),
};

const SIGNATURES: [Signature; 9] = [
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
        name: CONCAT,
        params: &[
            Param {
                name: "sections",
                kind: Kind::List(&Kind::Or(&[
                    Kind::String,
                    Kind::Named(SECTION_TYPENAME),
                ])),
            },
            Param {
                name: "separator",
                kind: Kind::Optional(&Kind::String),
            },
        ],
        returns: &[Kind::String],
        doc: "The sections joined into one string, in the order they are given, with `separator` between them; a newline when none is given. A string is a section carrying only `content`.",
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

const OUTPUT_FIELDS: [Field; 6] = [
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

const SECTION_FIELDS: [Field; 2] = [
    Field {
        name: CONTENT,
        kind: Kind::String,
        doc: "The text of the fragment, whatever produced it. Required.",
    },
    Field {
        name: WHEN,
        kind: Kind::Optional(&Kind::Boolean),
        doc: "Whether the section lands. Defaults to `true`, and only `false` leaves it out; the `content` is already built either way.",
    },
];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES)
        })
        .record(record(OUTPUT_TYPENAME, OUTPUT_DOC).fields(&OUTPUT_FIELDS))
        .record(record(FILE_TYPENAME, FILE_DOC))
        .record(record(SECTION_TYPENAME, SECTION_DOC).fields(&SECTION_FIELDS))
}
