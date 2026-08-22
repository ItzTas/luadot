#[cfg(feature = "meta")]
use super::super::signature::{Kind, Param, Signature};

pub const NAMESPACE: &str = "fs";

pub const EXISTS: &str = "exists";

pub const IS_DIR: &str = "is_dir";

pub const MKDIR: &str = "mkdir";

pub const LS: &str = "ls";

pub const RM: &str = "rm";

pub const READ: &str = "read";

pub const WRITE: &str = "write";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.fs";

#[cfg(feature = "meta")]
pub const DOC: &str = "The filesystem, with no directory of a template in the way: `~` and a relative path resolve against your home directory, an absolute one reaches anywhere. Nothing here takes a backup.";

#[cfg(feature = "meta")]
const PATH_PARAM: Param = Param {
    name: "path",
    kind: Kind::String,
};

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 7] = [
    Signature {
        name: EXISTS,
        params: &[PATH_PARAM],
        returns: &[Kind::Boolean],
        doc: "Whether something is there: a file, a directory, or a symlink whatever it points at.",
    },
    Signature {
        name: IS_DIR,
        params: &[PATH_PARAM],
        returns: &[Kind::Boolean],
        doc: "Whether a directory is there, through a symlink too.",
    },
    Signature {
        name: MKDIR,
        params: &[PATH_PARAM],
        returns: &[],
        doc: "Creates the directory and every one leading to it; one already there is fine.",
    },
    Signature {
        name: LS,
        params: &[PATH_PARAM],
        returns: &[Kind::List(&Kind::String)],
        doc: "The names inside a directory, sorted, files and directories alike.",
    },
    Signature {
        name: RM,
        params: &[PATH_PARAM],
        returns: &[Kind::Boolean],
        doc: "Removes a file, a symlink, or a directory with everything under it, and says whether something was there. Your home directory and what holds it are refused.",
    },
    Signature {
        name: READ,
        params: &[PATH_PARAM],
        returns: &[Kind::String],
        doc: "What the file holds.",
    },
    Signature {
        name: WRITE,
        params: &[
            PATH_PARAM,
            Param {
                name: "text",
                kind: Kind::String,
            },
        ],
        returns: &[],
        doc: "Writes the text over the file, creating the directories leading to it.",
    },
];
