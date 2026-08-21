#[cfg(feature = "meta")]
use super::super::constants::CALL_METHOD;
#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind, Param, Signature};

pub const NAMESPACE: &str = "class";

pub const GET: &str = "get";

pub const NAME: &str = "name";

pub const PROMPT: &str = "prompt";

pub const CHOICES: &str = "choices";

pub const DEFAULT: &str = "default";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.class";

#[cfg(feature = "meta")]
pub const DOC: &str = "The classes of the machine: questions it answers once, read back by every script. The answers live in luadot's state, per machine, out of the repository.";

#[cfg(feature = "meta")]
pub const CLASS_TYPENAME: &str = "ld.Class";

#[cfg(feature = "meta")]
pub const CLASS_DOC: &str = "A class declaration.";

#[cfg(feature = "meta")]
pub const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[Param {
        name: "class",
        kind: Kind::Named(CLASS_TYPENAME),
    }],
    returns: &[],
    doc: "Declares a question this machine answers once. In `config.lua` it waits for `bootstrap`, `clone` or `luadot class` to ask; anywhere else it asks straight away and saves the answer. Declaring the same name twice replaces the first declaration.",
};

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 1] = [Signature {
    name: GET,
    params: &[Param {
        name: "name",
        kind: Kind::String,
    }],
    returns: &[Kind::Optional(&Kind::String)],
    doc: "The answer this machine gave, `nil` when it gave none.",
}];

#[cfg(feature = "meta")]
pub const CLASS_FIELDS: [Field; 4] = [
    Field {
        name: NAME,
        kind: Kind::String,
        doc: "How the class is read and answered; no spaces. Required.",
    },
    Field {
        name: PROMPT,
        kind: Kind::Optional(&Kind::String),
        doc: "What the machine is asked. Defaults to `define the class <name>`.",
    },
    Field {
        name: CHOICES,
        kind: Kind::Optional(&Kind::Or(&[Kind::String, Kind::List(&Kind::String)])),
        doc: "Restricts the answer to that list; without it the answer is free text.",
    },
    Field {
        name: DEFAULT,
        kind: Kind::Optional(&Kind::String),
        doc: "The answer pressing enter accepts, one of the choices. It only fills the prompt: an unanswered class still reads as `nil`.",
    },
];
