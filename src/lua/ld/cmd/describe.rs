use tealr::TypeWalker;

use super::super::constants::{CALL_METHOD, STRING_INDEX};
use super::super::signature::{Collect, Describe, Field, Kind, Param, Signature};

const NAMESPACE_TYPENAME: &str = "ld.cmd";

const DOC: &str = "Runs commands and returns their standard output, trailing newline removed. A non-zero exit stops the script; standard error and standard input stay on the terminal. Slow: it belongs in `bootstrap.lua` or a setup script.";

const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[Param {
        name: "line",
        kind: Kind::String,
    }],
    returns: &[Kind::String],
    doc: "Hands the whole line to `sh`, so pipes, globs and redirection work, and returns what it printed.",
};

const FIELDS: [Field; 1] = [Field {
    name: STRING_INDEX,
    kind: Kind::Function(
        &[Param {
            name: "...",
            kind: Kind::Variadic(&Kind::String),
        }],
        &[Kind::String],
    ),
    doc: "Indexed by a program name, runs the program itself with no shell in the way, every argument literal, and returns what it printed: `ld.cmd.git(\"status\")`.",
}];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker.namespace(NAMESPACE_TYPENAME, DOC, |record| {
        record.fields(&FIELDS).call(&CALL)
    })
}
