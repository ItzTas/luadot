#[cfg(feature = "meta")]
use super::super::constants::{CALL_METHOD, CONFLICT_TYPENAME, LINK_MODE_TYPENAME};
#[cfg(feature = "meta")]
use super::super::signature::{Kind, Param, Signature};
use super::super::table::Setter;
use super::{
    autocommit, autopush, backup, backup_age, backup_dir, backup_keep, conflict, lfs, link,
    passphrase_warn, pkg_warn, repo_dir,
};

pub const NAMESPACE: &str = "opt";

pub const AUTOCOMMIT: &str = "autocommit";

pub const AUTOPUSH: &str = "autopush";

pub const BACKUP: &str = "backup";

pub const BACKUP_AGE: &str = "backup_age";

pub const SPAN_KIND: &str = "a span like \"30d\"";

pub const BACKUP_DIR: &str = "backup_dir";

pub const BACKUP_KEEP: &str = "backup_keep";

pub const CONFLICT: &str = "conflict";

pub const LFS: &str = "lfs";

pub const LINK: &str = "link";

pub const PASSPHRASE_WARN: &str = "passphrase_warn";

pub const PKG_WARN: &str = "pkg_warn";

pub const REPO_DIR: &str = "repo_dir";

pub const SETTERS: [(&str, Setter); 12] = [
    (AUTOCOMMIT, autocommit::set),
    (AUTOPUSH, autopush::set),
    (BACKUP, backup::set),
    (BACKUP_AGE, backup_age::set),
    (BACKUP_DIR, backup_dir::set),
    (BACKUP_KEEP, backup_keep::set),
    (CONFLICT, conflict::set),
    (LFS, lfs::set),
    (LINK, link::set),
    (PASSPHRASE_WARN, passphrase_warn::set),
    (PKG_WARN, pkg_warn::set),
    (REPO_DIR, repo_dir::set),
];

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.opt";

#[cfg(feature = "meta")]
pub const DOC: &str = "The options of a run. Each setter takes one value; called with a table, `ld.opt` sets every key the table carries.";

#[cfg(feature = "meta")]
pub const OPTIONS_TYPENAME: &str = "ld.Options";

#[cfg(feature = "meta")]
pub const OPTIONS_DOC: &str = "The table form of the options, `ld.opt({ link = \"symbolic\" })`: only the keys it carries are set.";

#[cfg(feature = "meta")]
const ENABLED: Param = Param {
    name: "enabled",
    kind: Kind::Boolean,
};

#[cfg(feature = "meta")]
const PATH: Param = Param {
    name: "path",
    kind: Kind::String,
};

#[cfg(feature = "meta")]
pub const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[Param {
        name: "options",
        kind: Kind::Named(OPTIONS_TYPENAME),
    }],
    returns: &[],
    doc: "Sets several options at once; only the keys it carries.",
};

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 12] = [
    Signature {
        name: AUTOCOMMIT,
        params: &[ENABLED],
        returns: &[],
        doc: "Whether `add` and `rm` commit what they staged. Defaults to `false`.",
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
        doc: "Whether a call is warned about where it is slow or has no effect. Defaults to `true`.",
    },
    Signature {
        name: REPO_DIR,
        params: &[PATH],
        returns: &[],
        doc: "The repository luadot manages, winning over the one `clone` left behind. `~` and a relative path resolve against your home directory.",
    },
];
