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

#[cfg(test)]
mod tests {
    use std::process::Command;

    use gix::tempfile::{AutoRemove, ContainingDirectory};

    use super::*;

    const ALONE: &str = "LUADOT_TEST_PANIC_HOOK";
    const TEST: &str = "git::locks::tests::a_panic_takes_the_lock_files_still_registered_with_it";

    fn ran_in_its_own_process() -> bool {
        if std::env::var_os(ALONE).is_some() {
            return false;
        }

        let output = Command::new(std::env::current_exe().unwrap())
            .args([TEST, "--exact", "--test-threads", "1"])
            .env(ALONE, "1")
            .output()
            .unwrap();

        let report = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "{report}{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(report.contains("1 passed"), "{report}");

        true
    }

    #[test]
    fn a_panic_takes_the_lock_files_still_registered_with_it() {
        if ran_in_its_own_process() {
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        catch_panics();

        let held = gix::tempfile::new(
            dir.path(),
            ContainingDirectory::Exists,
            AutoRemove::Tempfile,
        )
        .unwrap();
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);

        panic::catch_unwind(|| panic!("panicked while holding a lock")).unwrap_err();

        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
        drop(held);
    }
}
