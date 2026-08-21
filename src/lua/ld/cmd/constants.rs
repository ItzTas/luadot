#[cfg(feature = "meta")]
use super::super::constants::{CALL_METHOD, STRING_INDEX};
#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind, Param, Signature};

pub const NAMESPACE: &str = "cmd";

pub const SHELL: &str = "sh";

pub const SHELL_ARG: &str = "-c";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.cmd";

#[cfg(feature = "meta")]
pub const DOC: &str = "Runs commands and returns their standard output, trailing newline removed. A non-zero exit stops the script; standard error and standard input stay on the terminal. Slow: it belongs in `bootstrap.lua` or a setup script, and warns elsewhere.";

#[cfg(feature = "meta")]
pub const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[Param {
        name: "line",
        kind: Kind::String,
    }],
    returns: &[Kind::String],
    doc: "Hands the whole line to `sh`, so pipes, globs and redirection work, and returns what it printed.",
};

#[cfg(feature = "meta")]
pub const FIELDS: [Field; 1] = [Field {
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
