use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, Field, Kind, Param, Signature, record};
use super::constants::{
    ADD, ALT, APPLY, ARGS, BOOTSTRAP, CD, CLASS, CLONE, CONFIG, DIFF, EDIT, ENTRY, EXEC, GIT,
    HINTS, INIT, MV, NEW, PUSH, REKEY, RENDER, RESTORE, RM, SETUP, STATUS, SUMMARY, SYNC, TAKE,
    TOOL,
};
use crate::files::{FileStatus, Side};
use crate::lua::config::constants::{
    AFTER, BEFORE, CONTENT, DEFAULT, DIFF_STATES, DRIFTED, MODE, NAME, PATH, SIDE, SOURCE, STATE,
    SYSTEM, TEMPLATES, TOTAL,
};

const NAMESPACE_TYPENAME: &str = "ld.on";

const DOC: &str = "One call per command, taking a table: functions to run `before` and `after` the command, what its hints print, and what `status` and `diff` print. Every function registered for a moment runs, in the order it was registered; the rest is replaced by a later call, key by key. Every command is customized apart.";

const TMPL_TYPENAME: &str = "ld.on.tmpl";

const TMPL_DOC: &str = "The two `tmpl` actions, customized apart.";

const AROUND_TYPENAME: &str = "ld.Around";

const AROUND_DOC: &str = "A function to run before the command and one after it, and what its hints print. Whatever a function returns is written as a line; a function returning nothing writes nothing.";

const HINT_TYPENAME: &str = "ld.Hint";

const HINT_DOC: &str =
    "One of the lines a command writes to say which call comes next, as it hands it to `hints`.";

const DIFF_OPTIONS_TYPENAME: &str = "ld.DiffOptions";

const DIFF_OPTIONS_DOC: &str = "What `diff` prints and which program compares the two sides, and a function to run before and after it. Whatever a function returns is written as a line; a function returning nothing writes nothing.";

const STATUS_OPTIONS_TYPENAME: &str = "ld.StatusOptions";

const STATUS_OPTIONS_DOC: &str = "What `status` prints, and a function to run before and after it. Whatever a function returns is written as a line; a function returning nothing writes nothing.";

const DIFF_FILE_TYPENAME: &str = "ld.DiffFile";

const DIFF_FILE_DOC: &str = "A drifted file, as `diff` hands it to `entry` and `render`.";

const STATUS_FILE_TYPENAME: &str = "ld.StatusFile";

const STATUS_FILE_DOC: &str =
    "An inspected file, synced or not, as `status` hands it to `entry` and `render`.";

const DIFF_COUNTS_TYPENAME: &str = "ld.DiffCounts";

const DIFF_COUNTS_DOC: &str = "What `diff` hands to `summary`, once per side.";

const STATUS_COUNTS_TYPENAME: &str = "ld.StatusCounts";

const STATUS_COUNTS_DOC: &str = "What `status` hands to `summary`, once per side.";

const CONTENT_TYPENAME: &str = "ld.Content";

const CONTENT_DOC: &str = "The bytes of both sides of a drifted file.";

const MODE_TYPENAME: &str = "ld.Mode";

const MODE_DOC: &str =
    "The permission bits of both sides of a drifted file, as octal strings like `\"0644\"`.";

const SIDE_TYPENAME: &str = "ld.Side";

const SIDE_DOC: &str =
    "Which side reported a file: the repository, or the templates that generated it.";

const DIFF_STATE_TYPENAME: &str = "ld.DiffState";

const DIFF_STATE_DOC: &str = "Where a drifted file stands, as `diff` reports it.";

const STATUS_STATE_TYPENAME: &str = "ld.StatusState";

const STATUS_STATE_DOC: &str = "Where an inspected file stands, as `status` reports it.";

const COUNT_DOC: &str = "The files in that state.";

const SIDES: [Side; 2] = [Side::Repository, Side::Generated];

const STATUS_STATES: [FileStatus; 5] = [
    FileStatus::Synced,
    FileStatus::Missing,
    FileStatus::Unlinked,
    FileStatus::Differs,
    FileStatus::Unreadable,
];

const WORDS: Kind = Kind::Optional(&Kind::Or(&[Kind::String, Kind::List(&Kind::String)]));

const LINE: Kind = Kind::Optional(&Kind::String);

const MOMENT: Kind = Kind::Optional(&Kind::Or(&[Kind::Function(&[], &[LINE]), Kind::False]));

const HINT: Kind = Kind::Optional(&Kind::Or(&[
    Kind::Function(
        &[Param {
            name: "hint",
            kind: Kind::Named(HINT_TYPENAME),
        }],
        &[LINE],
    ),
    Kind::False,
]));

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

const AROUND_PARAMS: [Param; 1] = [Param {
    name: "options",
    kind: Kind::Named(AROUND_TYPENAME),
}];

const fn around(name: &'static str, doc: &'static str) -> Signature {
    Signature {
        name,
        params: &AROUND_PARAMS,
        returns: &[],
        doc,
    }
}

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 21] = [
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
    around(MV, "Runs a function before and after `mv`."),
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
    around(TAKE, "Runs a function before and after `take`."),
];

const TMPL_SIGNATURES: [Signature; 2] = [
    around(ALT, "Runs a function before and after `tmpl alt`."),
    around(NEW, "Runs a function before and after `tmpl new`."),
];

const AFTER_FIELD: Field = Field {
    name: AFTER,
    kind: MOMENT,
    doc: "Runs once the command is done; a command that fails stops before it. Calls add up, in order; `false` drops the functions registered so far.",
};

const BEFORE_FIELD: Field = Field {
    name: BEFORE,
    kind: MOMENT,
    doc: "Runs once `config.lua` ran, before the command does anything. Calls add up, in order; `false` drops the functions registered so far.",
};

const HINTS_FIELD: Field = Field {
    name: HINTS,
    kind: HINT,
    doc: "Runs for every hint the command would write, in place of it. `false` silences them, and `ld.opt.hints(false)` silences the hints of every command.",
};

const AROUND_FIELDS: [Field; 3] = [AFTER_FIELD, BEFORE_FIELD, HINTS_FIELD];

const DIFF_FIELDS: [Field; 8] = [
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
    HINTS_FIELD,
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

const STATUS_FIELDS: [Field; 6] = [
    AFTER_FIELD,
    BEFORE_FIELD,
    Field {
        name: ENTRY,
        kind: STATUS_ENTRY,
        doc: "Runs for every inspected file, synced ones included, in place of the line and the sections the command would have written. `false` silences them.",
    },
    HINTS_FIELD,
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

const HINT_FIELDS: [Field; 2] = [
    Field {
        name: NAME,
        kind: Kind::String,
        doc: "Which hint it is: for `status`, the section it opens, like `\"differs\"`.",
    },
    Field {
        name: DEFAULT,
        kind: Kind::String,
        doc: "The line it stands in for.",
    },
];

const FILE_FIELDS: [Field; 3] = [
    Field {
        name: PATH,
        kind: Kind::String,
        doc: "The path as you use it: `.bashrc`, and `.netrc` for a secret the repository keeps as `.netrc.age`.",
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

const DIFF_FILE_FIELDS: [Field; 3] = [
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

const STATUS_FILE_FIELDS: [Field; 1] = [Field {
    name: STATE,
    kind: Kind::Named(STATUS_STATE_TYPENAME),
    doc: "Where the file stands.",
}];

const SIDE_FIELDS: [Field; 2] = [
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

const COUNTS_FIELDS: [Field; 3] = [
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

const DIFF_COUNTS_FIELDS: [Field; 1] = [Field {
    name: DRIFTED,
    kind: Kind::Integer,
    doc: "The files that differ.",
}];

const STATUS_COUNTS_FIELDS: [Field; 1] = [Field {
    name: TEMPLATES,
    kind: Kind::Integer,
    doc: "The templates behind the files, on the generated side.",
}];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .choices(SIDE_TYPENAME, SIDE_DOC, SIDES.iter().map(|side| side.dir()))
        .choices(
            DIFF_STATE_TYPENAME,
            DIFF_STATE_DOC,
            DIFF_STATES.iter().map(|(name, _)| *name),
        )
        .choices(
            STATUS_STATE_TYPENAME,
            STATUS_STATE_DOC,
            STATUS_STATES.iter().map(|state| state.name()),
        )
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES)
        })
        .namespace(TMPL_TYPENAME, TMPL_DOC, |record| {
            record.functions(&TMPL_SIGNATURES)
        })
        .record(record(AROUND_TYPENAME, AROUND_DOC).fields(&AROUND_FIELDS))
        .record(record(HINT_TYPENAME, HINT_DOC).fields(&HINT_FIELDS))
        .record(record(DIFF_OPTIONS_TYPENAME, DIFF_OPTIONS_DOC).fields(&DIFF_FIELDS))
        .record(record(STATUS_OPTIONS_TYPENAME, STATUS_OPTIONS_DOC).fields(&STATUS_FIELDS))
        .record(
            record(DIFF_FILE_TYPENAME, DIFF_FILE_DOC)
                .fields(&FILE_FIELDS)
                .fields(&DIFF_FILE_FIELDS),
        )
        .record(
            record(STATUS_FILE_TYPENAME, STATUS_FILE_DOC)
                .fields(&FILE_FIELDS)
                .fields(&STATUS_FILE_FIELDS),
        )
        .record(record(CONTENT_TYPENAME, CONTENT_DOC).fields(&SIDE_FIELDS))
        .record(record(MODE_TYPENAME, MODE_DOC).fields(&SIDE_FIELDS))
        .record(
            record(DIFF_COUNTS_TYPENAME, DIFF_COUNTS_DOC)
                .fields(&COUNTS_FIELDS)
                .fields(&DIFF_COUNTS_FIELDS),
        )
        .record(
            record(STATUS_COUNTS_TYPENAME, STATUS_COUNTS_DOC)
                .fields(&COUNTS_FIELDS)
                .fields(&STATUS_COUNTS_FIELDS)
                .fields(&counts()),
        )
}

fn counts() -> Vec<Field> {
    STATUS_STATES
        .iter()
        .map(|state| Field {
            name: state.name(),
            kind: Kind::Integer,
            doc: COUNT_DOC,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::command::Command;
    use super::super::constants::{AROUND_KEYS, DIFF_KEYS, STATUS_KEYS, TMPL};
    use super::*;

    #[test]
    fn every_command_is_described() {
        let mut described: Vec<String> = SIGNATURES
            .iter()
            .map(|signature| signature.name.to_string())
            .chain(
                TMPL_SIGNATURES
                    .iter()
                    .map(|signature| format!("{TMPL}.{}", signature.name)),
            )
            .collect();
        described.sort();

        let mut paths: Vec<String> = Command::ALL.iter().map(|command| command.path()).collect();
        paths.sort();

        assert_eq!(described, paths);
    }

    #[test]
    fn every_around_key_is_described() {
        let described: Vec<&str> = AROUND_FIELDS.iter().map(|field| field.name).collect();

        assert_eq!(described, AROUND_KEYS);
    }

    #[test]
    fn every_diff_key_is_described() {
        let described: Vec<&str> = DIFF_FIELDS.iter().map(|field| field.name).collect();

        assert_eq!(described, DIFF_KEYS);
    }

    #[test]
    fn every_status_key_is_described() {
        let described: Vec<&str> = STATUS_FIELDS.iter().map(|field| field.name).collect();

        assert_eq!(described, STATUS_KEYS);
    }
}
