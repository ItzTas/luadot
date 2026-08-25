use std::sync::Once;

use super::constants::PASSPHRASE_WARNING;
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
