use std::fmt::Display;

use super::constants::{GAP, PREFIX};

pub fn notice(message: impl Display) -> String {
    format!("{PREFIX}: {message}")
}

pub fn column(text: impl Display, width: usize) -> String {
    let text = text.to_string();
    if text.chars().count() >= width {
        return format!("{text}{GAP}");
    }

    format!("{text:<width$}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notice_prefixes_the_application_name() {
        assert_eq!(notice("nothing to apply"), "luadot: nothing to apply");
    }

    #[test]
    fn column_pads_the_text_to_the_width() {
        assert_eq!(column("synced", 9), "synced   ");
    }

    #[test]
    fn column_keeps_text_wider_than_the_column_and_separates_it() {
        assert_eq!(column("repository", 4), "repository  ");
    }

    #[test]
    fn column_separates_text_that_fills_the_column_exactly() {
        assert_eq!(column("synced", 6), "synced  ");
    }
}
