use super::constants::MILLIS;
use crate::utils::span;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Retention {
    keep: Option<u32>,
    age: Option<u64>,
}

impl Retention {
    pub fn new(keep: Option<u32>, age: Option<u64>) -> Self {
        Self { keep, age }
    }

    pub fn keep(&self) -> Option<u32> {
        self.keep
    }

    pub fn age(&self) -> Option<u64> {
        self.age
    }

    pub fn is_empty(&self) -> bool {
        self.keep.is_none() && self.age.is_none()
    }

    pub(super) fn extra(&self, taken: usize) -> usize {
        self.keep
            .map_or(0, |keep| taken.saturating_sub(keep as usize))
    }

    pub(super) fn cutoff(&self, now: u64) -> Option<u64> {
        self.age
            .map(|age| now.saturating_sub(age.saturating_mul(MILLIS)))
    }

    pub(super) fn label(&self) -> String {
        match (self.keep, self.age) {
            (Some(keep), Some(age)) => format!(
                "keeping the {keep} most recent taken in the last {}",
                span(age)
            ),
            (Some(keep), None) => format!("keeping the {keep} most recent"),
            (None, Some(age)) => format!("keeping the ones taken in the last {}", span(age)),
            (None, None) => "keeping every one of them".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_leaves_everything_beyond_it_over() {
        let retention = Retention::new(Some(2), None);

        assert!(!retention.is_empty());
        assert_eq!(retention.extra(5), 3);
        assert_eq!(retention.extra(1), 0);
        assert_eq!(retention.cutoff(1_000_000), None);
    }

    #[test]
    fn an_age_becomes_the_oldest_stamp_still_kept() {
        let retention = Retention::new(None, Some(60));

        assert!(!retention.is_empty());
        assert_eq!(retention.extra(5), 0);
        assert_eq!(retention.cutoff(1_000_000), Some(940_000));
        assert_eq!(retention.cutoff(1_000), Some(0));
    }
}
