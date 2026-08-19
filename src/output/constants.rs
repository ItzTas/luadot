use std::time::Duration;

use anstyle::{AnsiColor, Style};

use super::tone::Tone;

pub const PREFIX: &str = "luadot";

pub const PROGRESS_FRAME_RATE: f32 = 6.0;

pub const PROGRESS_DELAY: Duration = Duration::from_millis(200);

pub const LABEL_WIDTH: usize = 11;

pub const FIELD_WIDTH: usize = 12;

pub const ITEM_WIDTH: usize = 13;

pub const ITEM_INDENT: usize = 8;

pub const HINT_INDENT: usize = 2;

pub const GAP: &str = "  ";

pub const STYLES: [(Tone, Style); 5] = [
    (Tone::Good, AnsiColor::Green.on_default()),
    (Tone::Warning, AnsiColor::Yellow.on_default()),
    (Tone::Bad, AnsiColor::Red.on_default()),
    (Tone::Strong, Style::new().bold()),
    (Tone::Muted, Style::new().dimmed()),
];
