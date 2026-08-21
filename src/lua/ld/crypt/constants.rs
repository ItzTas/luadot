#[cfg(feature = "meta")]
use super::super::constants::{BACKEND_TYPENAME, CALL_METHOD, INTEGER_INDEX};
#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind as Shape, Param, Signature};
use super::super::table::Setter;
use super::{backend, lock};

pub const NAMESPACE: &str = "crypt";

pub const BACKEND: &str = "backend";

pub const LOCK: &str = "lock";

pub const LOCK_KEYS: &str = "crypt.lock";

pub const LOCK_KIND: &str = "\"passphrase\" or a table of `recipients` and `identity`";

pub const PASSPHRASE: &str = "passphrase";

pub const IDENTITY: &str = "identity";

pub const RECIPIENTS: &str = "recipients";

pub const SETTERS: [(&str, Setter); 2] = [(BACKEND, backend::set), (LOCK, lock::set)];

pub const TYPE: &str = "type";

pub const FILE: &str = "file";

pub const COMMAND: &str = "command";

pub const IDENTITY_KEYS: &str = "crypt.lock.identity";

pub const IDENTITY_TYPE: &str = "identity type";

pub const IDENTITY_KIND: &str =
    "a path, a command line, or a table carrying `type` and what it names";

pub const FILE_ALONE: &str = "`ld.crypt.lock`'s identity of type `file` takes one path";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    File,
    Command,
}

pub const IDENTITY_TYPES: [(&str, Kind); 2] = [(COMMAND, Kind::Command), (FILE, Kind::File)];

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.crypt";

#[cfg(feature = "meta")]
pub const DOC: &str = "How managed secrets are encrypted. It has an effect in `config.lua` only; elsewhere a call does nothing and says so.";

#[cfg(feature = "meta")]
pub const OPTIONS_TYPENAME: &str = "ld.CryptOptions";

#[cfg(feature = "meta")]
pub const OPTIONS_DOC: &str =
    "The table form, `ld.crypt({ backend = \"gpg\" })`: only the keys it carries are set.";

#[cfg(feature = "meta")]
pub const KEYS_TYPENAME: &str = "ld.Keys";

#[cfg(feature = "meta")]
pub const KEYS_DOC: &str =
    "A lock made of keys: who the files are encrypted to, and what decrypts them.";

#[cfg(feature = "meta")]
pub const IDENTITY_TYPENAME: &str = "ld.Identity";

#[cfg(feature = "meta")]
pub const IDENTITY_DOC: &str = "An identity spelled out: its words are a path, or a program and its arguments run without a shell, and `type` says which when the guess would be wrong.";

#[cfg(feature = "meta")]
pub const IDENTITY_KIND_TYPENAME: &str = "ld.IdentityType";

#[cfg(feature = "meta")]
pub const IDENTITY_KIND_DOC: &str = "Whether an identity names a file or a command.";

#[cfg(feature = "meta")]
pub const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[Param {
        name: "options",
        kind: Shape::Named(OPTIONS_TYPENAME),
    }],
    returns: &[],
    doc: "Sets several crypt options at once; only the keys it carries.",
};

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 2] = [
    Signature {
        name: BACKEND,
        params: &[Param {
            name: "name",
            kind: Shape::Named(BACKEND_TYPENAME),
        }],
        returns: &[],
        doc: "Tool used to encrypt and decrypt managed files. Defaults to `\"age\"`.",
    },
    Signature {
        name: LOCK,
        params: &[Param {
            name: "lock",
            kind: Shape::Or(&[Shape::Word(PASSPHRASE), Shape::Named(KEYS_TYPENAME)]),
        }],
        returns: &[],
        doc: "How secrets are locked: the word locks with a passphrase, the table with keys. Defaults to keys with none set.",
    },
];

#[cfg(feature = "meta")]
pub const KEYS_FIELDS: [Field; 2] = [
    Field {
        name: RECIPIENTS,
        kind: Shape::Optional(&Shape::Or(&[Shape::String, Shape::List(&Shape::String)])),
        doc: "Public keys, or key ids for gpg, the files are encrypted to.",
    },
    Field {
        name: IDENTITY,
        kind: Shape::Optional(&Shape::Or(&[
            Shape::String,
            Shape::Named(IDENTITY_TYPENAME),
        ])),
        doc: "What decrypts with age; gpg uses its keyring. A path resolves `~` and a relative path against your home directory; a command line prints the key instead, and a string carrying a space is read as one.",
    },
];

#[cfg(feature = "meta")]
pub const IDENTITY_FIELDS: [Field; 2] = [
    Field {
        name: TYPE,
        kind: Shape::Optional(&Shape::Named(IDENTITY_KIND_TYPENAME)),
        doc: "Says outright whether the words name a file or a command.",
    },
    Field {
        name: INTEGER_INDEX,
        kind: Shape::String,
        doc: "The path, or the program and its arguments.",
    },
];
