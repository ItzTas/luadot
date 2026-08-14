use std::fmt::Display;

use crate::files::SyncOutcome;
use crate::output::{self, Tone};

const LABELS: [(SyncOutcome, &str, Tone); 4] = [
    (SyncOutcome::Created, "create", Tone::Good),
    (SyncOutcome::Replaced, "replace", Tone::Warning),
    (SyncOutcome::Skipped, "skip", Tone::Muted),
    (SyncOutcome::AlreadySynced, "", Tone::Muted),
];

pub fn preview(outcome: SyncOutcome, path: impl Display) {
    let (tone, label) = display(outcome);
    if label.is_empty() {
        return;
    }

    output::entry(tone, label, path);
}

fn display(outcome: SyncOutcome) -> (Tone, &'static str) {
    LABELS
        .iter()
        .find(|(kind, _, _)| *kind == outcome)
        .map(|(_, label, tone)| (*tone, *label))
        .unwrap_or((Tone::Muted, ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_change_carries_a_label() {
        for outcome in [
            SyncOutcome::Created,
            SyncOutcome::Replaced,
            SyncOutcome::Skipped,
        ] {
            let (_, label) = display(outcome);
            assert!(!label.is_empty());
        }
    }

    #[test]
    fn an_unchanged_file_is_not_reported() {
        let (_, label) = display(SyncOutcome::AlreadySynced);

        assert!(label.is_empty());
    }

    #[test]
    fn labels_fit_the_printed_column() {
        for (_, label, _) in LABELS {
            assert!(label.len() < 9);
        }
    }
}
