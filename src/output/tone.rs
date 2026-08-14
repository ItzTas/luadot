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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tone_has_a_style() {
        for tone in [
            Tone::Good,
            Tone::Warning,
            Tone::Bad,
            Tone::Strong,
            Tone::Muted,
        ] {
            assert_ne!(tone.style(), Style::new(), "expected {tone:?} to be styled");
        }
    }

    #[test]
    fn every_tone_has_its_own_style() {
        for (index, (tone, style)) in STYLES.iter().enumerate() {
            let duplicate = STYLES
                .iter()
                .skip(index + 1)
                .find(|(_, other)| other == style);

            assert!(duplicate.is_none(), "expected {tone:?} to be distinct");
        }
    }
}
