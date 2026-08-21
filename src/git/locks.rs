use std::io;
use std::panic;

use super::constants::INTERRUPT_GRACE;

pub fn guard() -> io::Result<()> {
    catch_panics();
    unsafe { gix::interrupt::init_handler(INTERRUPT_GRACE, || {}) }.map(|_| ())
}

fn catch_panics() {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        gix::tempfile::registry::cleanup_tempfiles_signal_safe();
        previous(info);
    }));
}
