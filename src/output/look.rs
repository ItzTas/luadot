use anstyle::{Color, Effects, Style};

use super::tone::Tone;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Look {
    tone: Option<Tone>,
    fg: Option<Color>,
    bg: Option<Color>,
    bold: Option<bool>,
    dim: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
}

impl From<Tone> for Look {
    fn from(tone: Tone) -> Self {
        Self {
            tone: Some(tone),
            ..Self::default()
        }
    }
}

impl Look {
    pub fn with_tone(mut self, tone: Option<Tone>) -> Self {
        self.tone = tone.or(self.tone);
        self
    }

    pub fn with_fg(mut self, fg: Option<Color>) -> Self {
        self.fg = fg.or(self.fg);
        self
    }

    pub fn with_bg(mut self, bg: Option<Color>) -> Self {
        self.bg = bg.or(self.bg);
        self
    }

    pub fn with_bold(mut self, bold: Option<bool>) -> Self {
        self.bold = bold.or(self.bold);
        self
    }

    pub fn with_dim(mut self, dim: Option<bool>) -> Self {
        self.dim = dim.or(self.dim);
        self
    }

    pub fn with_italic(mut self, italic: Option<bool>) -> Self {
        self.italic = italic.or(self.italic);
        self
    }

    pub fn with_underline(mut self, underline: Option<bool>) -> Self {
        self.underline = underline.or(self.underline);
        self
    }

    pub fn style(self) -> Style {
        let mut style = self.tone.map(Tone::style).unwrap_or_default();
        if let Some(fg) = self.fg {
            style = style.fg_color(Some(fg));
        }
        if let Some(bg) = self.bg {
            style = style.bg_color(Some(bg));
        }

        for (wanted, effect) in [
            (self.bold, Effects::BOLD),
            (self.dim, Effects::DIMMED),
            (self.italic, Effects::ITALIC),
            (self.underline, Effects::UNDERLINE),
        ] {
            style = applied(style, wanted, effect);
        }

        style
    }
}

fn applied(style: Style, wanted: Option<bool>, effect: Effects) -> Style {
    match wanted {
        None => style,
        Some(true) => style.effects(style.get_effects().insert(effect)),
        Some(false) => style.effects(style.get_effects().remove(effect)),
    }
}

#[cfg(test)]
mod tests {
    use anstyle::{AnsiColor, RgbColor};

    use super::*;

    #[test]
    fn a_color_wins_over_the_color_of_the_tone() {
        let look = Look::from(Tone::Good).with_fg(Some(AnsiColor::Red.into()));

        assert_eq!(look.style().get_fg_color(), Some(AnsiColor::Red.into()));
    }

    #[test]
    fn every_effect_reaches_the_style() {
        let style = Look::default()
            .with_bold(Some(true))
            .with_dim(Some(true))
            .with_italic(Some(true))
            .with_underline(Some(true))
            .with_bg(Some(RgbColor(255, 136, 0).into()))
            .style();

        for effect in [
            Effects::BOLD,
            Effects::DIMMED,
            Effects::ITALIC,
            Effects::UNDERLINE,
        ] {
            assert!(style.get_effects().contains(effect));
        }
        assert_eq!(style.get_bg_color(), Some(RgbColor(255, 136, 0).into()));
    }
}
