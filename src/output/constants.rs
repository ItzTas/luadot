use anstyle::{AnsiColor, Style};

use super::tone::Tone;

pub const PREFIX: &str = "luadot";

pub const LABEL_WIDTH: usize = 9;

pub const FIELD_WIDTH: usize = 12;

pub const STYLES: [(Tone, Style); 5] = [
    (Tone::Good, AnsiColor::Green.on_default()),
    (Tone::Warning, AnsiColor::Yellow.on_default()),
    (Tone::Bad, AnsiColor::Red.on_default()),
    (Tone::Strong, Style::new().bold()),
    (Tone::Muted, Style::new().dimmed()),
];
