use std::fmt::Display;

use super::constants::GAP;

pub fn notice(message: impl Display) -> String {
    format!("luadot: {message}")
}

pub fn column(text: impl Display, width: usize) -> String {
    let text = text.to_string();
    if text.chars().count() >= width {
        return format!("{text}{GAP}");
    }

    format!("{text:<width$}")
}
