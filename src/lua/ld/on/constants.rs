use mlua::{Function, Lua};

#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind, Param, Signature};
use super::command::Command;
use super::{around, diff, status};
#[cfg(feature = "meta")]
use crate::files::{FileStatus, Side};
use crate::lua::config::constants::{AFTER, BEFORE};
#[cfg(feature = "meta")]
use crate::lua::config::constants::{
    CONTENT, DEFAULT, DRIFTED, MODE, PATH, SIDE, SOURCE, STATE, SYSTEM, TEMPLATES, TOTAL,
};

pub const NAMESPACE: &str = "on";

pub const ADD: &str = "add";

pub const APPLY: &str = "apply";

pub const BOOTSTRAP: &str = "bootstrap";

pub const CD: &str = "cd";

pub const CLASS: &str = "class";

pub const CLONE: &str = "clone";

pub const CONFIG: &str = "config";

pub const DIFF: &str = "diff";

pub const EDIT: &str = "edit";

pub const EXEC: &str = "exec";

pub const GIT: &str = "git";

pub const INIT: &str = "init";

pub const PUSH: &str = "push";

pub const REKEY: &str = "rekey";

pub const RESTORE: &str = "restore";

pub const RM: &str = "rm";

pub const SETUP: &str = "setup";

pub const STATUS: &str = "status";

pub const SYNC: &str = "sync";

pub const TMPL: &str = "tmpl";

pub const ALT: &str = "alt";

pub const NEW: &str = "new";

pub const TMPL_ALT: &str = "tmpl alt";

pub const TMPL_NEW: &str = "tmpl new";

pub type Customizer = fn(&Lua, Command) -> mlua::Result<Function>;

pub const FUNCTIONS: [(&str, Command, Customizer); 19] = [
    (ADD, Command::Add, around::function),
    (APPLY, Command::Apply, around::function),
    (BOOTSTRAP, Command::Bootstrap, around::function),
    (CD, Command::Cd, around::function),
    (CLASS, Command::Class, around::function),
    (CLONE, Command::Clone, around::function),
    (CONFIG, Command::Config, around::function),
    (DIFF, Command::Diff, diff::function),
    (EDIT, Command::Edit, around::function),
    (EXEC, Command::Exec, around::function),
    (GIT, Command::Git, around::function),
    (INIT, Command::Init, around::function),
    (PUSH, Command::Push, around::function),
    (REKEY, Command::Rekey, around::function),
    (RESTORE, Command::Restore, around::function),
    (RM, Command::Rm, around::function),
    (SETUP, Command::Setup, around::function),
    (STATUS, Command::Status, status::function),
    (SYNC, Command::Sync, around::function),
];

pub const TMPL_FUNCTIONS: [(&str, Command, Customizer); 2] = [
    (ALT, Command::TmplAlt, around::function),
    (NEW, Command::TmplNew, around::function),
];

pub const ARGS: &str = "args";

pub const ENTRY: &str = "entry";

pub const RENDER: &str = "render";

pub const SUMMARY: &str = "summary";

pub const TOOL: &str = "tool";

pub const AROUND_KEYS: [&str; 2] = [AFTER, BEFORE];

pub const DIFF_KEYS: [&str; 7] = [AFTER, ARGS, BEFORE, ENTRY, RENDER, SUMMARY, TOOL];

pub const STATUS_KEYS: [&str; 5] = [AFTER, BEFORE, ENTRY, RENDER, SUMMARY];

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.on";

#[cfg(feature = "meta")]
pub const DOC: &str = "One call per command, taking a table: functions to run `before` and `after` the command, and what `status` and `diff` print. Every function registered for a moment runs, in the order it was registered; what `status` and `diff` print is replaced by a later call, key by key. Every command is customized apart.";

#[cfg(feature = "meta")]
pub const TMPL_TYPENAME: &str = "ld.on.tmpl";

#[cfg(feature = "meta")]
pub const TMPL_DOC: &str = "The two `tmpl` actions, customized apart.";

#[cfg(feature = "meta")]
pub const AROUND_TYPENAME: &str = "ld.Around";

#[cfg(feature = "meta")]
pub const AROUND_DOC: &str = "A function to run before the command and one after it. Whatever a function returns is written as a line; a function returning nothing writes nothing.";

#[cfg(feature = "meta")]
pub const DIFF_OPTIONS_TYPENAME: &str = "ld.DiffOptions";

#[cfg(feature = "meta")]
pub const DIFF_OPTIONS_DOC: &str = "What `diff` prints and which program compares the two sides, and a function to run before and after it. Whatever a function returns is written as a line; a function returning nothing writes nothing.";

#[cfg(feature = "meta")]
pub const STATUS_OPTIONS_TYPENAME: &str = "ld.StatusOptions";

#[cfg(feature = "meta")]
pub const STATUS_OPTIONS_DOC: &str = "What `status` prints, and a function to run before and after it. Whatever a function returns is written as a line; a function returning nothing writes nothing.";

#[cfg(feature = "meta")]
pub const DIFF_FILE_TYPENAME: &str = "ld.DiffFile";

#[cfg(feature = "meta")]
pub const DIFF_FILE_DOC: &str = "A drifted file, as `diff` hands it to `entry` and `render`.";

#[cfg(feature = "meta")]
pub const STATUS_FILE_TYPENAME: &str = "ld.StatusFile";

#[cfg(feature = "meta")]
pub const STATUS_FILE_DOC: &str =
    "An inspected file, synced or not, as `status` hands it to `entry` and `render`.";

#[cfg(feature = "meta")]
pub const DIFF_COUNTS_TYPENAME: &str = "ld.DiffCounts";

#[cfg(feature = "meta")]
pub const DIFF_COUNTS_DOC: &str = "What `diff` hands to `summary`, once per side.";

#[cfg(feature = "meta")]
pub const STATUS_COUNTS_TYPENAME: &str = "ld.StatusCounts";

#[cfg(feature = "meta")]
pub const STATUS_COUNTS_DOC: &str = "What `status` hands to `summary`, once per side.";

#[cfg(feature = "meta")]
pub const CONTENT_TYPENAME: &str = "ld.Content";

#[cfg(feature = "meta")]
pub const CONTENT_DOC: &str = "The bytes of both sides of a drifted file.";

#[cfg(feature = "meta")]
pub const MODE_TYPENAME: &str = "ld.Mode";

#[cfg(feature = "meta")]
pub const MODE_DOC: &str =
    "The permission bits of both sides of a drifted file, as octal strings like `\"0644\"`.";

#[cfg(feature = "meta")]
pub const SIDE_TYPENAME: &str = "ld.Side";

#[cfg(feature = "meta")]
pub const SIDE_DOC: &str =
    "Which side reported a file: the repository, or the templates that generated it.";

#[cfg(feature = "meta")]
pub const DIFF_STATE_TYPENAME: &str = "ld.DiffState";

#[cfg(feature = "meta")]
pub const DIFF_STATE_DOC: &str = "Where a drifted file stands, as `diff` reports it.";

#[cfg(feature = "meta")]
pub const STATUS_STATE_TYPENAME: &str = "ld.StatusState";

#[cfg(feature = "meta")]
pub const STATUS_STATE_DOC: &str = "Where an inspected file stands, as `status` reports it.";

#[cfg(feature = "meta")]
pub const COUNT_DOC: &str = "The files in that state.";

#[cfg(feature = "meta")]
pub const SIDES: [Side; 2] = [Side::Repository, Side::Generated];

#[cfg(feature = "meta")]
pub const STATUS_STATES: [FileStatus; 5] = [
    FileStatus::Synced,
    FileStatus::Missing,
    FileStatus::Unlinked,
    FileStatus::Differs,
    FileStatus::Unreadable,
];

#[cfg(feature = "meta")]
const WORDS: Kind = Kind::Optional(&Kind::Or(&[Kind::String, Kind::List(&Kind::String)]));

#[cfg(feature = "meta")]
const LINE: Kind = Kind::Optional(&Kind::String);

#[cfg(feature = "meta")]
const MOMENT: Kind = Kind::Optional(&Kind::Or(&[Kind::Function(&[], &[LINE]), Kind::False]));

#[cfg(feature = "meta")]
const DIFF_ENTRY: Kind = Kind::Optional(&Kind::Or(&[
    Kind::Function(
        &[Param {
            name: "file",
            kind: Kind::Named(DIFF_FILE_TYPENAME),
        }],
        &[LINE],
    ),
    Kind::False,
]));

#[cfg(feature = "meta")]
const DIFF_RENDER: Kind = Kind::Optional(&Kind::Or(&[
    Kind::Function(
        &[Param {
            name: "files",
            kind: Kind::List(&Kind::Named(DIFF_FILE_TYPENAME)),
        }],
        &[LINE],
    ),
    Kind::False,
]));

#[cfg(feature = "meta")]
const DIFF_SUMMARY: Kind = Kind::Optional(&Kind::Or(&[
    Kind::Function(
        &[Param {
            name: "counts",
            kind: Kind::Named(DIFF_COUNTS_TYPENAME),
        }],
        &[LINE],
    ),
    Kind::String,
    Kind::False,
]));

#[cfg(feature = "meta")]
const STATUS_ENTRY: Kind = Kind::Optional(&Kind::Or(&[
    Kind::Function(
        &[Param {
            name: "file",
            kind: Kind::Named(STATUS_FILE_TYPENAME),
        }],
        &[LINE],
    ),
    Kind::False,
]));

#[cfg(feature = "meta")]
const STATUS_RENDER: Kind = Kind::Optional(&Kind::Or(&[
    Kind::Function(
        &[Param {
            name: "files",
            kind: Kind::List(&Kind::Named(STATUS_FILE_TYPENAME)),
        }],
        &[LINE],
    ),
    Kind::False,
]));

#[cfg(feature = "meta")]
const STATUS_SUMMARY: Kind = Kind::Optional(&Kind::Or(&[
    Kind::Function(
        &[Param {
            name: "counts",
            kind: Kind::Named(STATUS_COUNTS_TYPENAME),
        }],
        &[LINE],
    ),
    Kind::String,
    Kind::False,
]));

#[cfg(feature = "meta")]
const AROUND_PARAMS: [Param; 1] = [Param {
    name: "options",
    kind: Kind::Named(AROUND_TYPENAME),
}];

#[cfg(feature = "meta")]
const fn around(name: &'static str, doc: &'static str) -> Signature {
    Signature {
        name,
        params: &AROUND_PARAMS,
        returns: &[],
        doc,
    }
}

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 19] = [
    around(ADD, "Runs a function before and after `add`."),
    around(APPLY, "Runs a function before and after `apply`."),
    around(BOOTSTRAP, "Runs a function before and after `bootstrap`."),
    around(CD, "Runs a function before and after `cd`."),
    around(CLASS, "Runs a function before and after `class`."),
    around(CLONE, "Runs a function before and after `clone`."),
    around(CONFIG, "Runs a function before and after `config`."),
    Signature {
        name: DIFF,
        params: &[Param {
            name: "options",
            kind: Kind::Named(DIFF_OPTIONS_TYPENAME),
        }],
        returns: &[],
        doc: "Says what `diff` prints and which program compares the two sides, and runs a function before and after it.",
    },
    around(EDIT, "Runs a function before and after `edit`."),
    around(EXEC, "Runs a function before and after `exec`."),
    around(GIT, "Runs a function before and after `git`."),
    around(INIT, "Runs a function before and after `init`."),
    around(PUSH, "Runs a function before and after `push`."),
    around(REKEY, "Runs a function before and after `rekey`."),
    around(RESTORE, "Runs a function before and after `restore`."),
    around(RM, "Runs a function before and after `rm`."),
    around(SETUP, "Runs a function before and after `setup`."),
    Signature {
        name: STATUS,
        params: &[Param {
            name: "options",
            kind: Kind::Named(STATUS_OPTIONS_TYPENAME),
        }],
        returns: &[],
        doc: "Says what `status` prints, line by line, and runs a function before and after it.",
    },
    around(SYNC, "Runs a function before and after `sync`."),
];

#[cfg(feature = "meta")]
pub const TMPL_SIGNATURES: [Signature; 2] = [
    around(ALT, "Runs a function before and after `tmpl alt`."),
    around(NEW, "Runs a function before and after `tmpl new`."),
];

#[cfg(feature = "meta")]
const AFTER_FIELD: Field = Field {
    name: AFTER,
    kind: MOMENT,
    doc: "Runs once the command is done; a command that fails stops before it. Calls add up, in order; `false` drops the functions registered so far.",
};

#[cfg(feature = "meta")]
const BEFORE_FIELD: Field = Field {
    name: BEFORE,
    kind: MOMENT,
    doc: "Runs once `config.lua` ran, before the command does anything. Calls add up, in order; `false` drops the functions registered so far.",
};

#[cfg(feature = "meta")]
pub const AROUND_FIELDS: [Field; 2] = [AFTER_FIELD, BEFORE_FIELD];

#[cfg(feature = "meta")]
pub const DIFF_FIELDS: [Field; 7] = [
    AFTER_FIELD,
    Field {
        name: ARGS,
        kind: WORDS,
        doc: "Extra arguments for whichever program compares the two sides; right after `diff` when git runs.",
    },
    BEFORE_FIELD,
    Field {
        name: ENTRY,
        kind: DIFF_ENTRY,
        doc: "Runs for every drifted file, in place of the line the command would have written. `false` silences the line.",
    },
    Field {
        name: RENDER,
        kind: DIFF_RENDER,
        doc: "Runs once, with every drifted file, and takes the whole report over; nothing is compared afterwards. `false` reports the files without diffing them.",
    },
    Field {
        name: SUMMARY,
        kind: DIFF_SUMMARY,
        doc: "Replaces the line each side opens with; a string stands as it is, `false` silences it.",
    },
    Field {
        name: TOOL,
        kind: WORDS,
        doc: "The program comparing the two sides instead of `git diff`, with its arguments; it gets the two sides as two directories, its last two arguments. Exit status 0 or 1 counts as success.",
    },
];

#[cfg(feature = "meta")]
pub const STATUS_FIELDS: [Field; 5] = [
    AFTER_FIELD,
    BEFORE_FIELD,
    Field {
        name: ENTRY,
        kind: STATUS_ENTRY,
        doc: "Runs for every inspected file, synced ones included, in place of the line and the sections the command would have written. `false` silences them.",
    },
    Field {
        name: RENDER,
        kind: STATUS_RENDER,
        doc: "Runs once, with every inspected file, and takes the whole report over.",
    },
    Field {
        name: SUMMARY,
        kind: STATUS_SUMMARY,
        doc: "Replaces the line each side opens with; a string stands as it is, `false` silences it.",
    },
];

#[cfg(feature = "meta")]
pub const FILE_FIELDS: [Field; 3] = [
    Field {
        name: PATH,
        kind: Kind::String,
        doc: "The path as the repository writes it: `.bashrc`.",
    },
    Field {
        name: SYSTEM,
        kind: Kind::String,
        doc: "The absolute path of the system copy.",
    },
    Field {
        name: SIDE,
        kind: Kind::Named(SIDE_TYPENAME),
        doc: "`\"repository\"` for a managed file, `\"generated\"` for one a template produced.",
    },
];

#[cfg(feature = "meta")]
pub const DIFF_FILE_FIELDS: [Field; 3] = [
    Field {
        name: STATE,
        kind: Kind::Named(DIFF_STATE_TYPENAME),
        doc: "Where the file stands.",
    },
    Field {
        name: CONTENT,
        kind: Kind::Named(CONTENT_TYPENAME),
        doc: "The bytes of both sides.",
    },
    Field {
        name: MODE,
        kind: Kind::Named(MODE_TYPENAME),
        doc: "The permission bits of both sides.",
    },
];

#[cfg(feature = "meta")]
pub const STATUS_FILE_FIELDS: [Field; 1] = [Field {
    name: STATE,
    kind: Kind::Named(STATUS_STATE_TYPENAME),
    doc: "Where the file stands.",
}];

#[cfg(feature = "meta")]
pub const SIDE_FIELDS: [Field; 2] = [
    Field {
        name: SOURCE,
        kind: Kind::String,
        doc: "The repository's side.",
    },
    Field {
        name: SYSTEM,
        kind: Kind::Optional(&Kind::String),
        doc: "The system's side; absent when the file is not there.",
    },
];

#[cfg(feature = "meta")]
pub const COUNTS_FIELDS: [Field; 3] = [
    Field {
        name: SIDE,
        kind: Kind::Named(SIDE_TYPENAME),
        doc: "The side the line opens.",
    },
    Field {
        name: TOTAL,
        kind: Kind::Integer,
        doc: "The files that side reported.",
    },
    Field {
        name: DEFAULT,
        kind: Kind::String,
        doc: "The line it stands in for.",
    },
];

#[cfg(feature = "meta")]
pub const DIFF_COUNTS_FIELDS: [Field; 1] = [Field {
    name: DRIFTED,
    kind: Kind::Integer,
    doc: "The files that differ.",
}];

#[cfg(feature = "meta")]
pub const STATUS_COUNTS_FIELDS: [Field; 1] = [Field {
    name: TEMPLATES,
    kind: Kind::Integer,
    doc: "The templates behind the files, on the generated side.",
}];
