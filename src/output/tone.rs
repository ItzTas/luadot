use anstyle::{AnsiColor, Style};

const STYLES: [(Tone, Style); 5] = [
    (Tone::Good, AnsiColor::Green.on_default()),
    (Tone::Warning, AnsiColor::Yellow.on_default()),
    (Tone::Bad, AnsiColor::Red.on_default()),
    (Tone::Strong, Style::new().bold()),
    (Tone::Muted, Style::new().dimmed()),
];

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
