use super::constants::{AGE, GPG};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Backend {
    #[default]
    Age,
    Gpg,
}

impl Backend {
    pub fn name(self) -> &'static str {
        match self {
            Self::Age => AGE,
            Self::Gpg => GPG,
        }
    }

    pub fn extension(self) -> &'static str {
        self.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_backend_names_the_tool_it_runs() {
        assert_eq!(Backend::Age.name(), "age");
        assert_eq!(Backend::Gpg.name(), "gpg");
    }
}
