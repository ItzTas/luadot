use tealr::TypeWalker;

use super::super::constants::{CALL_METHOD, CONFLICT_TYPENAME, LINK_MODE_TYPENAME};
use super::super::signature::{Collect, Describe, Kind, Param, Signature, record};
use super::constants::{
    AUTOCOMMIT, AUTOPUSH, BACKUP, BACKUP_AGE, BACKUP_DIR, BACKUP_KEEP, CONFLICT, HINTS, LFS, LINK,
    PASSPHRASE_WARN, PKG_WARN, REPO_DIR,
};

const NAMESPACE_TYPENAME: &str = "ld.opt";

const DOC: &str = "The options of a run. Each setter takes one value; called with a table, `ld.opt` sets every key the table carries.";

const OPTIONS_TYPENAME: &str = "ld.Options";

const OPTIONS_DOC: &str = "The table form of the options, `ld.opt({ link = \"symbolic\" })`: only the keys it carries are set.";

const ENABLED: Param = Param {
    name: "enabled",
    kind: Kind::Boolean,
};

const PATH: Param = Param {
    name: "path",
    kind: Kind::String,
};

const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[Param {
        name: "options",
        kind: Kind::Named(OPTIONS_TYPENAME),
    }],
    returns: &[],
    doc: "Sets several options at once; only the keys it carries.",
};

const SIGNATURES: [Signature; 13] = [
    Signature {
        name: AUTOCOMMIT,
        params: &[ENABLED],
        returns: &[],
        doc: "Whether `add`, `rm` and `mv` commit what they staged. Defaults to `false`.",
    },
    Signature {
        name: AUTOPUSH,
        params: &[ENABLED],
        returns: &[],
        doc: "Whether that commit is pushed too, committing first. Defaults to `false`.",
    },
    Signature {
        name: BACKUP,
        params: &[ENABLED],
        returns: &[],
        doc: "Whether a file is copied aside before luadot writes over it. Defaults to `true`.",
    },
    Signature {
        name: BACKUP_AGE,
        params: &[Param {
            name: "span",
            kind: Kind::String,
        }],
        returns: &[],
        doc: "How long a backup is kept, as a span like `\"30d\"` in `s`, `m`, `h`, `d` or `w`; the ones older than that are dropped. Defaults to keeping them forever.",
    },
    Signature {
        name: BACKUP_DIR,
        params: &[PATH],
        returns: &[],
        doc: "Where those copies land. `~` and a relative path resolve against your home directory. Defaults to `~/.local/share/luadot/backups`.",
    },
    Signature {
        name: BACKUP_KEEP,
        params: &[Param {
            name: "count",
            kind: Kind::Integer,
        }],
        returns: &[],
        doc: "How many backups to keep, one or more; the oldest ones are dropped once there are more. Defaults to keeping every one of them.",
    },
    Signature {
        name: CONFLICT,
        params: &[Param {
            name: "policy",
            kind: Kind::Named(CONFLICT_TYPENAME),
        }],
        returns: &[],
        doc: "Default answer when `apply` finds a differing file already on the system.",
    },
    Signature {
        name: HINTS,
        params: &[ENABLED],
        returns: &[],
        doc: "Whether a command writes the lines saying which call comes next. Defaults to `true`, and a `hints` given to `ld.on` wins over it.",
    },
    Signature {
        name: LFS,
        params: &[ENABLED],
        returns: &[],
        doc: "Whether luadot installs the Git LFS filters and writes the attributes the rules ask for. Defaults to `true`, and has no effect without `git-lfs` on your PATH.",
    },
    Signature {
        name: LINK,
        params: &[Param {
            name: "mode",
            kind: Kind::Named(LINK_MODE_TYPENAME),
        }],
        returns: &[],
        doc: "Default strategy used to link a managed file.",
    },
    Signature {
        name: PASSPHRASE_WARN,
        params: &[ENABLED],
        returns: &[],
        doc: "Whether passphrase mode says it is weaker than keys. Defaults to `true`.",
    },
    Signature {
        name: PKG_WARN,
        params: &[ENABLED],
        returns: &[],
        doc: "Whether a call is warned about where it has no effect. Defaults to `true`.",
    },
    Signature {
        name: REPO_DIR,
        params: &[PATH],
        returns: &[],
        doc: "The repository luadot manages, winning over the one `clone` left behind. `~` and a relative path resolve against your home directory.",
    },
];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES).call(&CALL)
        })
        .record(record(OPTIONS_TYPENAME, OPTIONS_DOC).options(&SIGNATURES))
}
