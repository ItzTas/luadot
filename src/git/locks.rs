use std::panic;

use super::constants::INTERRUPT_GRACE;

pub fn guard() {
    #[allow(unsafe_code)]
    let _ = unsafe { gix::interrupt::init_handler(INTERRUPT_GRACE, || {}) };

    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        gix::tempfile::registry::cleanup_tempfiles_signal_safe();
        previous(info);
    }));
}
