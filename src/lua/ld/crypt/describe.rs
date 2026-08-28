use tealr::TypeWalker;

use super::super::constants::{BACKEND_TYPENAME, CALL_METHOD, INTEGER_INDEX};
use super::super::signature::{Collect, Describe, Field, Kind as Shape, Param, Signature, record};
use super::constants::{BACKEND, IDENTITY, IDENTITY_TYPES, LOCK, PASSPHRASE, RECIPIENTS, TYPE};

const NAMESPACE_TYPENAME: &str = "ld.crypt";

const DOC: &str = "How managed secrets are encrypted. It has an effect in `config.lua` only; elsewhere a call does nothing and says so.";

const OPTIONS_TYPENAME: &str = "ld.CryptOptions";

const OPTIONS_DOC: &str =
    "The table form, `ld.crypt({ backend = \"gpg\" })`: only the keys it carries are set.";

const KEYS_TYPENAME: &str = "ld.Keys";

const KEYS_DOC: &str =
    "A lock made of keys: who the files are encrypted to, and what decrypts them.";

const IDENTITY_TYPENAME: &str = "ld.Identity";

const IDENTITY_DOC: &str = "An identity spelled out: its words are a path, or a program and its arguments run without a shell, and `type` says which when the guess would be wrong.";

const IDENTITY_KIND_TYPENAME: &str = "ld.IdentityType";

const IDENTITY_KIND_DOC: &str = "Whether an identity names a file or a command.";

const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[Param {
        name: "options",
        kind: Shape::Named(OPTIONS_TYPENAME),
    }],
    returns: &[],
    doc: "Sets several crypt options at once; only the keys it carries.",
};

const SIGNATURES: [Signature; 2] = [
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

const KEYS_FIELDS: [Field; 2] = [
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

const IDENTITY_FIELDS: [Field; 2] = [
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

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .choices(
            IDENTITY_KIND_TYPENAME,
            IDENTITY_KIND_DOC,
            IDENTITY_TYPES.iter().map(|(name, _)| *name),
        )
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES).call(&CALL)
        })
        .record(record(OPTIONS_TYPENAME, OPTIONS_DOC).options(&SIGNATURES))
        .record(record(KEYS_TYPENAME, KEYS_DOC).fields(&KEYS_FIELDS))
        .record(record(IDENTITY_TYPENAME, IDENTITY_DOC).fields(&IDENTITY_FIELDS))
}
