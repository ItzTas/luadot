use super::super::constants::{CONFLICT, LINK, MATCH, MODE, ON_CHANGE, REGEX};
#[cfg(feature = "meta")]
use super::super::constants::{CONFLICT_TYPENAME, LINK_MODE_TYPENAME};
#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind, Param, Signature};
#[cfg(feature = "meta")]
use crate::lua::bundled::lpeg::{LPEG_MODULE, RE_MODULE};

pub const RULES: &str = "rules";

pub const IGNORE: &str = "ignore";

pub const OWNER: &str = "owner";

pub const ENCRYPT: &str = "encrypt";

pub const LFS: &str = "lfs";

pub const AUTOCOMMIT: &str = "autocommit";

pub const AUTOPUSH: &str = "autopush";

pub const RULE_KEYS: [&str; 12] = [
    MATCH, REGEX, LINK, CONFLICT, ON_CHANGE, IGNORE, MODE, OWNER, ENCRYPT, LFS, AUTOCOMMIT,
    AUTOPUSH,
];

pub const TASK: &str = "task";

pub const ABOUT: &str = "about";

pub const RUN: &str = "run";

pub const TASK_KEYS: [&str; 2] = [ABOUT, RUN];

pub const BUILTINS: [&str; 26] = [
    "init",
    "clone",
    "add",
    "rm",
    "status",
    "diff",
    "apply",
    "tmpl",
    "restore",
    "edit",
    "rekey",
    "exec",
    "config",
    "class",
    "bootstrap",
    "setup",
    "task",
    "cd",
    "sync",
    "git",
    "push",
    "doc",
    "meta",
    "completions",
    "man",
    "help",
];

#[cfg(feature = "meta")]
pub const API_DOC: &str = "The interface luadot installs in every script it runs: `config.lua`, `bootstrap.lua`, the setup scripts, the templates and `luadot exec`. A call does the same thing wherever it runs, on the one configuration the command is using.";

#[cfg(feature = "meta")]
pub const RULE_TYPENAME: &str = "ld.Rule";

#[cfg(feature = "meta")]
pub const RULE_DOC: &str = "A rule: the files it covers through `match` or `regex`, never both, and what it sets for them. A pattern naming a directory covers everything under it.";

#[cfg(feature = "meta")]
const PATTERNS: Kind = Kind::Optional(&Kind::Or(&[Kind::String, Kind::List(&Kind::String)]));

#[cfg(feature = "meta")]
pub const FIELDS: [Field; 2] = [
    Field {
        name: LPEG_MODULE,
        kind: Kind::Table,
        doc: "The LPeg module, the table `require(\"lpeg\")` returns, loaded when first reached.",
    },
    Field {
        name: RE_MODULE,
        kind: Kind::Table,
        doc: "LPeg's `re` module, the table `require(\"re\")` returns, loaded when first reached.",
    },
];

#[cfg(feature = "meta")]
pub const TASK_TYPENAME: &str = "ld.Task";

#[cfg(feature = "meta")]
pub const TASK_DOC: &str = "A command of the configuration's own, as `ld.task` takes it.";

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 2] = [
    Signature {
        name: RULES,
        params: &[Param {
            name: "rules",
            kind: Kind::Or(&[
                Kind::Named(RULE_TYPENAME),
                Kind::List(&Kind::Named(RULE_TYPENAME)),
            ]),
        }],
        returns: &[],
        doc: "Overrides `link` and `conflict` for the files a glob or a regular expression matches, names an `on_change` command for them, sets the `mode` and `owner` they are placed with, marks them as never managed, marks them as encrypted, stores them in Git LFS, and commits and pushes them on their own. A single rule needs no list around it. Calls accumulate, and the last matching rule wins, key by key.",
    },
    Signature {
        name: TASK,
        params: &[
            Param {
                name: "name",
                kind: Kind::String,
            },
            Param {
                name: "task",
                kind: Kind::Named(TASK_TYPENAME),
            },
        ],
        returns: &[],
        doc: "Registers a command of the configuration's own: `luadot <name>` and `luadot task <name>` run its function with the arguments that follow. The name of a command luadot already has is refused, and so is one registered twice. Only `config.lua` registers; elsewhere the call does nothing and says so.",
    },
];

#[cfg(feature = "meta")]
pub const TASK_FIELDS: [Field; 2] = [
    Field {
        name: ABOUT,
        kind: Kind::Optional(&Kind::String),
        doc: "One line saying what the task does, shown by `luadot task --list`.",
    },
    Field {
        name: RUN,
        kind: Kind::Function(
            &[Param {
                name: "argv",
                kind: Kind::List(&Kind::String),
            }],
            &[Kind::Optional(&Kind::String)],
        ),
        doc: "What runs, handed everything after the task name. Whatever it returns is written as a line; an error stops the command. Required.",
    },
];

#[cfg(feature = "meta")]
pub const RULE_FIELDS: [Field; 12] = [
    Field {
        name: MATCH,
        kind: PATTERNS,
        doc: "A glob relative to the repository root, or a table of them: `*` matches within a segment, `**` crosses segments.",
    },
    Field {
        name: REGEX,
        kind: PATTERNS,
        doc: "A regular expression in Rust's syntax, or a table of them, matched against the path as written with no anchoring of its own.",
    },
    Field {
        name: LINK,
        kind: Kind::Optional(&Kind::Named(LINK_MODE_TYPENAME)),
        doc: "How the matching files are placed.",
    },
    Field {
        name: CONFLICT,
        kind: Kind::Optional(&Kind::Named(CONFLICT_TYPENAME)),
        doc: "Answer when the system copy differs.",
    },
    Field {
        name: ON_CHANGE,
        kind: Kind::Optional(&Kind::String),
        doc: "A command line that runs after `apply` or `tmpl alt` created or replaced one of those files.",
    },
    Field {
        name: IGNORE,
        kind: Kind::Optional(&Kind::Boolean),
        doc: "Whether the matching files are left unmanaged.",
    },
    Field {
        name: MODE,
        kind: Kind::Optional(&Kind::String),
        doc: "Three or four octal digits, the permission bits a matching file is placed with, and put back when they drift. An encrypted file carries `600` without it.",
    },
    Field {
        name: OWNER,
        kind: Kind::Optional(&Kind::String),
        doc: "`\"user\"` or `\"user:group\"`, who owns a matching file once placed, set through `chown`.",
    },
    Field {
        name: ENCRYPT,
        kind: Kind::Optional(&Kind::Boolean),
        doc: "Whether `add` stores the matching files encrypted.",
    },
    Field {
        name: LFS,
        kind: Kind::Optional(&Kind::Boolean),
        doc: "Whether the matching files are stored in Git LFS. Needs `match`, since git attributes have no regular expressions, and does not go with `encrypt`. luadot writes the patterns into the repository's `.local/share/luadot/git/attributes`, between the `# luadot:lfs` markers, and copies that file into `.git/info/attributes`.",
    },
    Field {
        name: AUTOCOMMIT,
        kind: Kind::Optional(&Kind::Boolean),
        doc: "Whether `add` and `rm` commit on their own once one of those files is staged.",
    },
    Field {
        name: AUTOPUSH,
        kind: Kind::Optional(&Kind::Boolean),
        doc: "Whether that commit is pushed too. It commits on its own, so `autocommit` comes with it, and `autocommit = false` holds both back.",
    },
];
