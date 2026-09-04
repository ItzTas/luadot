use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, Kind, Param, Signature};
use super::constants::{ESCAPE, FIND, GMATCH, GSUB, MATCH, SPLIT, TEST};

const NAMESPACE_TYPENAME: &str = "ld.regex";

const DOC: &str = "Regular expressions in Rust's syntax, the engine the `regex` rule key uses: linear time, no backreferences or lookaround. Lua strings eat one backslash, so `\\d` is written `\"\\\\d\"`.";

const TEXT_PARAM: Param = Param {
    name: "text",
    kind: Kind::String,
};

const PATTERN_PARAM: Param = Param {
    name: "pattern",
    kind: Kind::String,
};

const LIMIT_PARAM: Param = Param {
    name: "limit",
    kind: Kind::Optional(&Kind::Integer),
};

const GROUPS: Kind = Kind::Variadic(&Kind::Optional(&Kind::String));

const SIGNATURES: [Signature; 7] = [
    Signature {
        name: TEST,
        params: &[TEXT_PARAM, PATTERN_PARAM],
        returns: &[Kind::Boolean],
        doc: "Whether the expression matches anywhere in the text.",
    },
    Signature {
        name: MATCH,
        params: &[TEXT_PARAM, PATTERN_PARAM],
        returns: &[GROUPS],
        doc: "The whole match, then each of its groups, `nil` for a group that did not take part; nothing when the expression does not match.",
    },
    Signature {
        name: FIND,
        params: &[TEXT_PARAM, PATTERN_PARAM],
        returns: &[
            Kind::Optional(&Kind::Integer),
            Kind::Optional(&Kind::Integer),
        ],
        doc: "Where the match starts and where it ends, counted from 1 like `string.find`; nothing when the expression does not match.",
    },
    Signature {
        name: GMATCH,
        params: &[TEXT_PARAM, PATTERN_PARAM],
        returns: &[Kind::Function(&[], &[GROUPS])],
        doc: "An iterator walking every match, each one yielding the whole match then its groups.",
    },
    Signature {
        name: GSUB,
        params: &[
            TEXT_PARAM,
            PATTERN_PARAM,
            Param {
                name: "replacement",
                kind: Kind::Or(&[
                    Kind::String,
                    Kind::Function(
                        &[Param {
                            name: "...",
                            kind: GROUPS,
                        }],
                        &[Kind::Optional(&Kind::String)],
                    ),
                ]),
            },
            LIMIT_PARAM,
        ],
        returns: &[Kind::String, Kind::Integer],
        doc: "The text with the matches rewritten, and how many were. A string carries the groups as `$1` or `${name}`; a function receives what `match` yields and returns the piece to write, or `nil` to leave that match alone.",
    },
    Signature {
        name: SPLIT,
        params: &[TEXT_PARAM, PATTERN_PARAM, LIMIT_PARAM],
        returns: &[Kind::List(&Kind::String)],
        doc: "The pieces the expression cuts the text into; with a limit, the last piece keeps the rest.",
    },
    Signature {
        name: ESCAPE,
        params: &[TEXT_PARAM],
        returns: &[Kind::String],
        doc: "The text as an expression matching itself, every special character quoted.",
    },
];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker.namespace(NAMESPACE_TYPENAME, DOC, |record| {
        record.functions(&SIGNATURES)
    })
}
