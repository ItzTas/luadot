use tealr::TypeWalker;

use super::super::constants::CALL_METHOD;
use super::super::signature::{Collect, Describe, Field, Kind, Param, Signature, record};
use super::constants::{
    BG, BOLD, COLORS, DIM, ENTRY, ERROR, FG, FIELD, INDENT, ITALIC, MARK, NEWLINE, NOTE, SECTION,
    STREAM, STREAMS, TIME, TONE, TONES, UNDERLINE, WARN, WIDTH,
};

const NAMESPACE_TYPENAME: &str = "ld.print";

const DOC: &str = "Writes lines the way luadot writes them, styled by the table beside the text. Every color is dropped when the output is not a terminal.";

const OPTIONS_TYPENAME: &str = "ld.PrintOptions";

const OPTIONS_DOC: &str = "The table beside the text, styling the line.";

const TONE_TYPENAME: &str = "ld.Tone";

const TONE_DOC: &str = "The palette luadot's own output uses.";

const STREAM_TYPENAME: &str = "ld.Stream";

const STREAM_DOC: &str = "Where a line goes.";

const COLOR_TYPENAME: &str = "ld.Color";

const COLOR_DOC: &str = "The sixteen ANSI color names. A color is also a number from 0 to 255, or a hex color like `\"#ff8800\"`.";

const TEXT: Kind = Kind::Or(&[Kind::String, Kind::Number]);

const COLOR: Kind = Kind::Optional(&Kind::Or(&[
    Kind::Named(COLOR_TYPENAME),
    Kind::Integer,
    Kind::String,
]));

const EFFECT: Kind = Kind::Optional(&Kind::Boolean);

const EFFECT_DOC: &str = "Adds the attribute, or takes back one the tone carries.";

const OPTIONS_PARAM: Param = Param {
    name: "options",
    kind: Kind::Optional(&Kind::Named(OPTIONS_TYPENAME)),
};

const TEXT_PARAM: Param = Param {
    name: "text",
    kind: TEXT,
};

const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[TEXT_PARAM, OPTIONS_PARAM],
    returns: &[],
    doc: "Writes a line to the terminal, styled the way the options ask.",
};

const SIGNATURES: [Signature; 6] = [
    Signature {
        name: ENTRY,
        params: &[
            Param {
                name: "label",
                kind: TEXT,
            },
            TEXT_PARAM,
            OPTIONS_PARAM,
        ],
        returns: &[],
        doc: "The label in a column of its own and the text beside it.",
    },
    Signature {
        name: ERROR,
        params: &[TEXT_PARAM, OPTIONS_PARAM],
        returns: &[],
        doc: "`luadot: text`, in red, on the error stream.",
    },
    Signature {
        name: FIELD,
        params: &[
            Param {
                name: "name",
                kind: TEXT,
            },
            Param {
                name: "value",
                kind: TEXT,
            },
            OPTIONS_PARAM,
        ],
        returns: &[],
        doc: "A name in a column of its own and the value it holds beside it.",
    },
    Signature {
        name: NOTE,
        params: &[TEXT_PARAM, OPTIONS_PARAM],
        returns: &[],
        doc: "`luadot: text`.",
    },
    Signature {
        name: SECTION,
        params: &[
            Param {
                name: "title",
                kind: TEXT,
            },
            OPTIONS_PARAM,
        ],
        returns: &[],
        doc: "A blank line and the title, in bold.",
    },
    Signature {
        name: WARN,
        params: &[TEXT_PARAM, OPTIONS_PARAM],
        returns: &[],
        doc: "`luadot: text`, in yellow, on the error stream.",
    },
];

const OPTIONS_FIELDS: [Field; 13] = [
    Field {
        name: BG,
        kind: COLOR,
        doc: "The color behind the text, over whatever the tone carries.",
    },
    Field {
        name: BOLD,
        kind: EFFECT,
        doc: EFFECT_DOC,
    },
    Field {
        name: DIM,
        kind: EFFECT,
        doc: EFFECT_DOC,
    },
    Field {
        name: FG,
        kind: COLOR,
        doc: "The color of the text, over whatever the tone carries.",
    },
    Field {
        name: INDENT,
        kind: Kind::Optional(&Kind::Integer),
        doc: "Spaces before everything else.",
    },
    Field {
        name: ITALIC,
        kind: EFFECT,
        doc: EFFECT_DOC,
    },
    Field {
        name: MARK,
        kind: Kind::Optional(&Kind::Or(&[
            Kind::String,
            Kind::Function(&[], &[Kind::String]),
        ])),
        doc: "What opens the line, one space before the text; a function is called every time the line is written.",
    },
    Field {
        name: NEWLINE,
        kind: Kind::Optional(&Kind::Boolean),
        doc: "Whether the line ends; `false` leaves the cursor where the text stopped.",
    },
    Field {
        name: STREAM,
        kind: Kind::Optional(&Kind::Named(STREAM_TYPENAME)),
        doc: "Where the line goes. Defaults to `\"stdout\"`.",
    },
    Field {
        name: TIME,
        kind: Kind::Optional(&Kind::Or(&[Kind::Boolean, Kind::String])),
        doc: "A timestamp opening the line, before the `mark`: `true` for `%H:%M:%S`, or a strftime format like `\"%H:%M\"`.",
    },
    Field {
        name: TONE,
        kind: Kind::Optional(&Kind::Named(TONE_TYPENAME)),
        doc: "The palette luadot's own output uses.",
    },
    Field {
        name: UNDERLINE,
        kind: EFFECT,
        doc: EFFECT_DOC,
    },
    Field {
        name: WIDTH,
        kind: Kind::Optional(&Kind::Integer),
        doc: "The column the styled part is padded to.",
    },
];

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .choices(TONE_TYPENAME, TONE_DOC, TONES.iter().map(|(name, _)| *name))
        .choices(
            STREAM_TYPENAME,
            STREAM_DOC,
            STREAMS.iter().map(|(name, _)| *name),
        )
        .choices(
            COLOR_TYPENAME,
            COLOR_DOC,
            COLORS.iter().map(|(name, _)| *name),
        )
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES).call(&CALL)
        })
        .record(record(OPTIONS_TYPENAME, OPTIONS_DOC).fields(&OPTIONS_FIELDS))
}
