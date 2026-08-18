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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_lock_is_the_recipient_keys() {
        assert_eq!(Lock::default(), Lock::Keys);
        assert!(!Lock::default().passphrase());
    }

    #[test]
    fn the_option_picks_the_lock() {
        assert_eq!(Lock::of(true), Lock::Passphrase);
        assert_eq!(Lock::of(false), Lock::Keys);
        assert!(Lock::of(true).passphrase());
    }

    #[test]
    fn only_passphrase_mode_is_announced() {
        assert!(announced(Lock::Passphrase, true));
        assert!(!announced(Lock::Passphrase, false));
        assert!(!announced(Lock::Keys, true));
    }

    #[test]
    fn the_warning_names_the_way_out() {
        assert!(PASSPHRASE_WARNING.contains("`ld.crypt.passphrase_warn(false)`"));
    }

    #[test]
    fn resolving_answers_the_lock_the_configuration_asks_for() {
        assert_eq!(lock(false, true), Lock::Keys);
        assert_eq!(lock(true, false), Lock::Passphrase);
    }
}
