use anstyle::AnsiColor;

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
