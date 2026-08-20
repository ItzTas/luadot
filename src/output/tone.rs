use anstyle::Style;

use super::constants::STYLES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Good,
    Warning,
    Bad,
    Strong,
    Muted,
}

impl Tone {
    pub fn style(self) -> Style {
        STYLES
            .iter()
            .find(|(kind, _)| *kind == self)
            .map(|(_, style)| *style)
            .unwrap_or_default()
    }
}
