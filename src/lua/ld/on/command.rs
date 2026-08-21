use super::super::constants::API;
use super::constants::{
    ADD, ALT, APPLY, BOOTSTRAP, CD, CLASS, CLONE, CONFIG, DIFF, EDIT, EXEC, GIT, INIT, NAMESPACE,
    NEW, PUSH, REKEY, RESTORE, RM, SETUP, STATUS, SYNC, TMPL, TMPL_ALT, TMPL_NEW,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    Add,
    Apply,
    Bootstrap,
    Cd,
    Class,
    Clone,
    Config,
    Diff,
    Edit,
    Exec,
    Git,
    Init,
    Push,
    Rekey,
    Restore,
    Rm,
    Setup,
    Status,
    Sync,
    TmplAlt,
    TmplNew,
}

impl Command {
    pub fn name(self) -> &'static str {
        match self {
            Self::Add => ADD,
            Self::Apply => APPLY,
            Self::Bootstrap => BOOTSTRAP,
            Self::Cd => CD,
            Self::Class => CLASS,
            Self::Clone => CLONE,
            Self::Config => CONFIG,
            Self::Diff => DIFF,
            Self::Edit => EDIT,
            Self::Exec => EXEC,
            Self::Git => GIT,
            Self::Init => INIT,
            Self::Push => PUSH,
            Self::Rekey => REKEY,
            Self::Restore => RESTORE,
            Self::Rm => RM,
            Self::Setup => SETUP,
            Self::Status => STATUS,
            Self::Sync => SYNC,
            Self::TmplAlt => TMPL_ALT,
            Self::TmplNew => TMPL_NEW,
        }
    }

    pub fn path(self) -> String {
        match self {
            Self::TmplAlt => format!("{TMPL}.{ALT}"),
            Self::TmplNew => format!("{TMPL}.{NEW}"),
            direct => direct.name().to_string(),
        }
    }

    pub fn call(self) -> String {
        format!("{API}.{NAMESPACE}.{}", self.path())
    }
}
