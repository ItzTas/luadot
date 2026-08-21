use super::constants::{AFTER, BEFORE};
use super::custom::Custom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Moment {
    Before,
    After,
}

#[derive(Debug, Clone, Default)]
pub struct Around {
    before: Option<Custom>,
    after: Option<Custom>,
}

impl Moment {
    pub fn key(self) -> &'static str {
        match self {
            Self::Before => BEFORE,
            Self::After => AFTER,
        }
    }
}

impl Around {
    pub fn with_before(mut self, before: Option<Custom>) -> Self {
        self.before = before;
        self
    }

    pub fn with_after(mut self, after: Option<Custom>) -> Self {
        self.after = after;
        self
    }

    pub fn merge(&mut self, other: Around) {
        self.before = other.before.or(self.before.take());
        self.after = other.after.or(self.after.take());
    }

    pub fn get(&self, moment: Moment) -> Option<&Custom> {
        match moment {
            Moment::Before => self.before.as_ref(),
            Moment::After => self.after.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merging_only_replaces_the_moments_the_other_one_carries() {
        let mut around = Around::default()
            .with_before(Some(Custom::Silent))
            .with_after(Some(Custom::Text("first".to_string())));

        around.merge(Around::default().with_after(Some(Custom::Text("second".to_string()))));

        assert!(matches!(around.get(Moment::Before), Some(Custom::Silent)));
        assert!(matches!(around.get(Moment::After), Some(Custom::Text(text)) if text == "second"));
    }

    #[test]
    fn each_moment_is_read_by_the_key_the_configuration_writes() {
        assert_eq!(Moment::Before.key(), "before");
        assert_eq!(Moment::After.key(), "after");
    }
}
