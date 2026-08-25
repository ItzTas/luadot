use anstyle::AnsiColor;

#[cfg(feature = "meta")]
use super::super::constants::CALL_METHOD;
#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind, Param, Signature};
use super::super::table::Builder;
use super::{entry, error, field, note, section, warn};
use crate::output::{Stream, Tone};

pub const NAMESPACE: &str = "print";

pub const ENTRY: &str = "entry";

pub const ERROR: &str = "error";

pub const FIELD: &str = "field";

pub const NOTE: &str = "note";

pub const SECTION: &str = "section";

pub const WARN: &str = "warn";

pub const FUNCTIONS: [(&str, Builder); 6] = [
    (ENTRY, entry::function),
    (ERROR, error::function),
    (FIELD, field::function),
    (NOTE, note::function),
    (SECTION, section::function),
    (WARN, warn::function),
];

pub const BG: &str = "bg";

pub const BOLD: &str = "bold";

pub const DIM: &str = "dim";

pub const FG: &str = "fg";

pub const INDENT: &str = "indent";

pub const ITALIC: &str = "italic";

pub const MARK: &str = "mark";

pub const NEWLINE: &str = "newline";

pub const STREAM: &str = "stream";

pub const TIME: &str = "time";

pub const TONE: &str = "tone";

pub const UNDERLINE: &str = "underline";

pub const WIDTH: &str = "width";

pub const OPTIONS: [&str; 13] = [
    BG, BOLD, DIM, FG, INDENT, ITALIC, MARK, NEWLINE, STREAM, TIME, TONE, UNDERLINE, WIDTH,
];

pub const TONES: [(&str, Tone); 5] = [
    ("good", Tone::Good),
    ("warning", Tone::Warning),
    ("bad", Tone::Bad),
    ("strong", Tone::Strong),
    ("muted", Tone::Muted),
];

pub const STREAMS: [(&str, Stream); 2] = [("stdout", Stream::Stdout), ("stderr", Stream::Stderr)];

pub const COLORS: [(&str, AnsiColor); 16] = [
    ("black", AnsiColor::Black),
    ("red", AnsiColor::Red),
    ("green", AnsiColor::Green),
    ("yellow", AnsiColor::Yellow),
    ("blue", AnsiColor::Blue),
    ("magenta", AnsiColor::Magenta),
    ("cyan", AnsiColor::Cyan),
    ("white", AnsiColor::White),
    ("bright-black", AnsiColor::BrightBlack),
    ("bright-red", AnsiColor::BrightRed),
    ("bright-green", AnsiColor::BrightGreen),
    ("bright-yellow", AnsiColor::BrightYellow),
    ("bright-blue", AnsiColor::BrightBlue),
    ("bright-magenta", AnsiColor::BrightMagenta),
    ("bright-cyan", AnsiColor::BrightCyan),
    ("bright-white", AnsiColor::BrightWhite),
];

pub const HEX: &str = "#";

pub const HEX_DIGITS: usize = 6;

pub const SHADES: &str = "a number from 0 to 255";

pub const OS: &str = "os";

pub const DATE: &str = "date";

pub const TIME_FORMAT: &str = "%H:%M:%S";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.print";

#[cfg(feature = "meta")]
pub const DOC: &str = "Writes lines the way luadot writes them, styled by the table beside the text. Every color is dropped when the output is not a terminal.";

#[cfg(feature = "meta")]
pub const OPTIONS_TYPENAME: &str = "ld.PrintOptions";

#[cfg(feature = "meta")]
pub const OPTIONS_DOC: &str = "The table beside the text, styling the line.";

#[cfg(feature = "meta")]
pub const TONE_TYPENAME: &str = "ld.Tone";

#[cfg(feature = "meta")]
pub const TONE_DOC: &str = "The palette luadot's own output uses.";

#[cfg(feature = "meta")]
pub const STREAM_TYPENAME: &str = "ld.Stream";

#[cfg(feature = "meta")]
pub const STREAM_DOC: &str = "Where a line goes.";

#[cfg(feature = "meta")]
pub const COLOR_TYPENAME: &str = "ld.Color";

#[cfg(feature = "meta")]
pub const COLOR_DOC: &str = "The sixteen ANSI color names. A color is also a number from 0 to 255, or a hex color like `\"#ff8800\"`.";

#[cfg(feature = "meta")]
const TEXT: Kind = Kind::Or(&[Kind::String, Kind::Number]);

#[cfg(feature = "meta")]
const COLOR: Kind = Kind::Optional(&Kind::Or(&[
    Kind::Named(COLOR_TYPENAME),
    Kind::Integer,
    Kind::String,
]));

#[cfg(feature = "meta")]
const EFFECT: Kind = Kind::Optional(&Kind::Boolean);

#[cfg(feature = "meta")]
const EFFECT_DOC: &str = "Adds the attribute, or takes back one the tone carries.";

#[cfg(feature = "meta")]
const OPTIONS_PARAM: Param = Param {
    name: "options",
    kind: Kind::Optional(&Kind::Named(OPTIONS_TYPENAME)),
};

#[cfg(feature = "meta")]
const TEXT_PARAM: Param = Param {
    name: "text",
    kind: TEXT,
};

#[cfg(feature = "meta")]
pub const CALL: Signature = Signature {
    name: CALL_METHOD,
    params: &[TEXT_PARAM, OPTIONS_PARAM],
    returns: &[],
    doc: "Writes a line to the terminal, styled the way the options ask.",
};

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 6] = [
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

#[cfg(feature = "meta")]
pub const OPTIONS_FIELDS: [Field; 13] = [
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
