use tealr::TypeWalker;

use super::super::constants::CALL_METHOD;
use super::super::signature::{Collect, Describe, Field, Kind, Param, Signature, record};
use super::constants::{CHOICES, DEFAULT, GET, NAME, PROMPT};

const NAMESPACE_TYPENAME: &str = "ld.class";

const DOC: &str = "The classes of the machine: questions it answers once, read back by every script. The answers live in luadot's state, per machine, out of the repository.";

const CLASS_TYPENAME: &str = "ld.Class";

const CLASS_DOC: &str = "A class declaration.";

const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[Param {
        name: "class",
        kind: Kind::Named(CLASS_TYPENAME),
    }],
    returns: &[],
    doc: "Declares a question this machine answers once. In `config.lua` it waits for `bootstrap`, `clone` or `luadot class` to ask; anywhere else it asks straight away and saves the answer. Declaring the same name twice replaces the first declaration.",
};

const SIGNATURES: [Signature; 1] = [Signature {
    name: GET,
    params: &[Param {
        name: "name",
        kind: Kind::String,
    }],
    returns: &[Kind::Optional(&Kind::String)],
    doc: "The answer this machine gave, `nil` when it gave none.",
}];

const CLASS_FIELDS: [Field; 4] = [
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

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES).call(&CALL)
        })
        .record(record(CLASS_TYPENAME, CLASS_DOC).fields(&CLASS_FIELDS))
}
