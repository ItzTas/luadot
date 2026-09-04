use std::sync::Once;

const PASSPHRASE_WARNING: &str = "passphrase mode is weaker than keys: one passphrase opens every secret, everyone sharing the repository shares it, and changing it means re-encrypting everything (silence this with `ld.opt.passphrase_warn(false)`)";
use crate::output;

static WARNED: Once = Once::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lock {
    #[default]
    Keys,
    Passphrase,
}

impl Lock {
    pub fn of(passphrase: bool) -> Self {
        match passphrase {
            true => Self::Passphrase,
            false => Self::Keys,
        }
    }

    pub fn passphrase(self) -> bool {
        self == Self::Passphrase
    }
}

pub fn lock(passphrase: bool, warn: bool) -> Lock {
    let lock = Lock::of(passphrase);
    if announced(lock, warn) {
        WARNED.call_once(|| output::warn(PASSPHRASE_WARNING));
    }

    lock
}

fn announced(lock: Lock, warn: bool) -> bool {
    lock.passphrase() && warn
}
