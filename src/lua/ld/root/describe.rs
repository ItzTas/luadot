use tealr::TypeWalker;

use super::super::constants::{
    API, CONFLICT, CONFLICT_TYPENAME, LINK, LINK_MODE_TYPENAME, MATCH, MODE, ON_CHANGE, REGEX,
    TRACK_TYPENAME,
};
use super::super::signature::{Collect, Describe, Field, Kind, Param, Signature, record};
use super::super::surface::{SURFACES, Surface};
use super::constants::{
    ABOUT, AUTOCOMMIT, AUTOPUSH, ENCRYPT, LFS, OWNER, RULES, RUN, SURFACE, TASK, TRACK, WHOLE,
};
use crate::lua::bundled::lpeg::{LPEG_MODULE, RE_MODULE};

const API_DOC: &str = "The interface luadot installs in every script it runs: `config.lua`, `bootstrap.lua`, the setup scripts, the templates and `luadot exec`. A call does the same thing wherever it runs, on the one configuration the command is using.";

const RULE_TYPENAME: &str = "ld.Rule";

const RULE_DOC: &str = "A rule: the files it covers through `match` or `regex`, never both, and what it sets for them. A pattern naming a directory covers everything under it.";

const PATTERNS: Kind = Kind::Optional(&Kind::Or(&[Kind::String, Kind::List(&Kind::String)]));

const SURFACE_TYPENAME: &str = "ld.Surface";

const SURFACE_DOC: &str = "Which script is running: `config.lua`, `bootstrap.lua`, a setup script, a template's `luadot.lua`, a standalone `.luadot` file, or `luadot exec`.";

const FIELDS: [Field; 3] = [
    Field {
        name: SURFACE,
        kind: Kind::Named(SURFACE_TYPENAME),
        doc: "Which script is running. `config.lua` runs before every command, so expensive work belongs elsewhere; `bootstrap.lua` runs once.",
    },
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

const TASK_TYPENAME: &str = "ld.Task";

const TASK_DOC: &str = "A command of the configuration's own, as `ld.task` takes it.";

const SIGNATURES: [Signature; 2] = [
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
        doc: "Overrides `link` and `conflict` for the files a glob or a regular expression matches, names an `on_change` command for them, sets the `mode` and `owner` they are placed with, says how they are `track`ed, marks them as encrypted, stores them in Git LFS, commits and pushes them on their own, and places matching directories `whole`. A single rule needs no list around it. Calls accumulate, and the last matching rule wins, key by key.",
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

const TASK_FIELDS: [Field; 2] = [
    Field {
        name: ABOUT,
        kind: Kind::Optional(&Kind::String),
        doc: "One line saying what the task does, shown by `luadot task`.",
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

const RULE_FIELDS: [Field; 13] = [
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
        name: TRACK,
        kind: Kind::Optional(&Kind::Named(TRACK_TYPENAME)),
        doc: "How luadot picks the matching files up: `\"auto\"` adds them on its own when `luadot add` runs with no path, `\"manual\"` waits for an explicit `luadot add`, `\"never\"` leaves them unmanaged. `\"auto\"` needs a `match` pattern opening on a literal name, since that is where luadot looks.",
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
        doc: "Whether `add`, `rm` and `mv` commit on their own once one of those files is staged.",
    },
    Field {
        name: AUTOPUSH,
        kind: Kind::Optional(&Kind::Boolean),
        doc: "Whether that commit is pushed too. It commits on its own, so `autocommit` comes with it, and `autocommit = false` holds both back.",
    },
    Field {
        name: WHOLE,
        kind: Kind::Optional(&Kind::Boolean),
        doc: "Whether a matching directory is placed whole, as one symlink or one copy of the tree, instead of file by file. Takes `link` `\"symbolic\"` or `\"copy\"`, and every file inside is stored: nothing under the directory may be excluded, encrypted, or a template.",
    },
];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .choices(SURFACE_TYPENAME, SURFACE_DOC, SURFACES.map(Surface::name))
        .namespace(API, API_DOC, |record| {
            record.fields(&FIELDS).functions(&SIGNATURES)
        })
        .record(record(RULE_TYPENAME, RULE_DOC).fields(&RULE_FIELDS))
        .record(record(TASK_TYPENAME, TASK_DOC).fields(&TASK_FIELDS))
}

#[cfg(test)]
mod tests {
    use super::super::constants::TASK_KEYS;
    use super::*;

    #[test]
    fn every_task_key_is_described() {
        let described: Vec<&str> = TASK_FIELDS.iter().map(|field| field.name).collect();

        assert_eq!(described, TASK_KEYS);
    }
}
