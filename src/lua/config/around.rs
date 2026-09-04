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

#[derive(Debug, Clone, Default)]
pub struct Chain {
    before: Vec<Custom>,
    after: Vec<Custom>,
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

    pub fn get(&self, moment: Moment) -> Option<&Custom> {
        match moment {
            Moment::Before => self.before.as_ref(),
            Moment::After => self.after.as_ref(),
        }
    }
}

impl Chain {
    pub fn add(&mut self, around: Around) {
        register(&mut self.before, around.before);
        register(&mut self.after, around.after);
    }

    pub fn all(&self, moment: Moment) -> &[Custom] {
        match moment {
            Moment::Before => &self.before,
            Moment::After => &self.after,
        }
    }
}

fn register(kept: &mut Vec<Custom>, added: Option<Custom>) {
    match added {
        None => {}
        Some(Custom::Silent) => kept.clear(),
        Some(custom) => kept.push(custom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(text: &str) -> Option<Custom> {
        Some(Custom::Text(text.to_string()))
    }

    fn texts(chain: &Chain, moment: Moment) -> Vec<String> {
        chain
            .all(moment)
            .iter()
            .map(|custom| custom.shown("a hook", ()).unwrap().unwrap_or_default())
            .collect()
    }

    #[test]
    fn functions_are_kept_in_order() {
        let mut chain = Chain::default();

        chain.add(Around::default().with_before(text("first")));
        chain.add(
            Around::default()
                .with_before(text("second"))
                .with_after(text("done")),
        );

        assert_eq!(texts(&chain, Moment::Before), ["first", "second"]);
        assert_eq!(texts(&chain, Moment::After), ["done"]);
    }

    #[test]
    fn a_registration_only_sets_its_moments() {
        let mut chain = Chain::default();

        chain.add(Around::default().with_before(text("first")));
        chain.add(Around::default());
        chain.add(Around::default().with_after(text("done")));

        assert_eq!(texts(&chain, Moment::Before), ["first"]);
        assert_eq!(texts(&chain, Moment::After), ["done"]);
    }

    #[test]
    fn false_drops_what_was_registered() {
        let mut chain = Chain::default();

        chain.add(Around::default().with_before(text("first")));
        chain.add(Around::default().with_before(Some(Custom::Silent)));
        chain.add(Around::default().with_before(text("second")));

        assert_eq!(texts(&chain, Moment::Before), ["second"]);
    }

    #[test]
    fn each_moment_is_read_by_its_key() {
        assert_eq!(Moment::Before.key(), "before");
        assert_eq!(Moment::After.key(), "after");
    }
}
