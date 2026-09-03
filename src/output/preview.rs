use std::fmt::Display;
use std::sync::OnceLock;

use super::print::entry;
use super::tone::Tone;
use crate::files::SyncOutcome;

static UNCHANGED: OnceLock<bool> = OnceLock::new();

const LABELS: [(SyncOutcome, &str, &str, Tone); 4] = [
    (SyncOutcome::Created, "create", "created", Tone::Good),
    (SyncOutcome::Replaced, "replace", "replaced", Tone::Warning),
    (SyncOutcome::Skipped, "skip", "skipped", Tone::Muted),
    (
        SyncOutcome::AlreadySynced,
        "unchanged",
        "unchanged",
        Tone::Muted,
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Planned,
    Settled,
}

pub fn set_unchanged(shown: bool) {
    let _ = UNCHANGED.set(shown);
}

fn unchanged_shown() -> bool {
    UNCHANGED.get().copied().unwrap_or(false)
}

pub fn preview(outcome: SyncOutcome, path: impl Display) {
    announce(display(outcome, Phase::Planned), path);
}

pub fn report(outcome: SyncOutcome, path: impl Display) {
    if outcome == SyncOutcome::AlreadySynced && !unchanged_shown() {
        return;
    }

    announce(display(outcome, Phase::Settled), path);
}

fn announce((tone, label): (Tone, &'static str), path: impl Display) {
    if label.is_empty() {
        return;
    }

    entry(tone, label, path);
}

fn display(outcome: SyncOutcome, phase: Phase) -> (Tone, &'static str) {
    LABELS
        .iter()
        .find(|(kind, _, _, _)| *kind == outcome)
        .map(|(_, planned, settled, tone)| {
            let label = match phase {
                Phase::Planned => *planned,
                Phase::Settled => *settled,
            };

            (*tone, label)
        })
        .unwrap_or((Tone::Muted, ""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::constants::{GAP, LABEL_WIDTH};

    #[test]
    fn labels_fit_the_printed_column() {
        for (_, planned, settled, _) in LABELS {
            assert!(planned.len() + GAP.len() <= LABEL_WIDTH);
            assert!(settled.len() + GAP.len() <= LABEL_WIDTH);
        }
    }
}
