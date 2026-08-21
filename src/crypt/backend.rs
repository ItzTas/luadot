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
