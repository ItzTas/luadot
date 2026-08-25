use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::thread;

use anyhow::{Result, anyhow};

use super::backend::Backend;
use super::constants::DECRYPT_WIDTH;
use super::identity::Identity;
use super::lock::Lock;
use super::plugin::plugged;
use super::run::decrypt;

#[derive(Debug)]
pub struct Ahead {
    command: String,
    lock: Lock,
    identity: Identity,
    width: usize,
    pending: VecDeque<(Backend, PathBuf)>,
    ready: VecDeque<(PathBuf, Result<Vec<u8>>)>,
}

impl Ahead {
    pub fn new(
        command: &str,
        lock: Lock,
        identity: Identity,
        sources: Vec<(Backend, PathBuf)>,
    ) -> Self {
        Self {
            command: command.to_string(),
            lock,
            identity,
            width: 0,
            pending: sources.into(),
            ready: VecDeque::new(),
        }
    }

    pub fn take(&mut self, backend: Backend, source: &Path) -> Result<Vec<u8>> {
        if self.ready.is_empty() {
            self.fill()?;
        }

        let Some((file, contents)) = self.ready.pop_front() else {
            return self.alone(backend, source);
        };
        if file != source {
            self.stop();
            return self.alone(backend, source);
        }

        contents
    }

    fn stop(&mut self) {
        self.ready.clear();
        self.pending.clear();
        self.width = 1;
    }

    fn alone(&mut self, backend: Backend, source: &Path) -> Result<Vec<u8>> {
        let key = self.identity.path(&self.command)?;

        decrypt(&self.command, backend, self.lock, key, source)
    }

    fn fill(&mut self) -> Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        if self.width == 0 {
            let jobs = self.pending.len();
            let key = self.identity.path(&self.command)?;
            let cores = thread::available_parallelism().map_or(1, usize::from);
            self.width = width(self.lock, key, jobs, cores);
        }

        let taken = self.width.min(self.pending.len());
        let window: Vec<(Backend, PathBuf)> = self.pending.drain(..taken).collect();
        let key = self.identity.path(&self.command)?.map(Path::to_path_buf);
        let opened = decrypted(&self.command, self.lock, key.as_deref(), &window);

        for ((_, file), contents) in window.into_iter().zip(opened) {
            self.ready.push_back((file, contents));
        }

        Ok(())
    }
}

fn width(lock: Lock, identity: Option<&Path>, jobs: usize, cores: usize) -> usize {
    if jobs < 2 || lock.passphrase() || plugged(identity) {
        return 1;
    }

    cores.min(DECRYPT_WIDTH).min(jobs)
}

fn decrypted(
    command: &str,
    lock: Lock,
    identity: Option<&Path>,
    window: &[(Backend, PathBuf)],
) -> Vec<Result<Vec<u8>>> {
    if window.len() < 2 {
        return window
            .iter()
            .map(|(backend, file)| decrypt(command, *backend, lock, identity, file))
            .collect();
    }

    thread::scope(|scope| {
        let handles: Vec<_> = window
            .iter()
            .map(|(backend, file)| {
                scope.spawn(move || decrypt(command, *backend, lock, identity, file))
            })
            .collect();

        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Err(anyhow!("{command}: a decryption did not finish")))
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_ahead_only_without_a_prompt() {
        let dir = tempfile::tempdir().unwrap();
        let key = dir.path().join("key.txt");
        let plugin = dir.path().join("plugin.txt");
        std::fs::write(&key, "AGE-SECRET-KEY-1TEST\n").unwrap();
        std::fs::write(&plugin, "AGE-PLUGIN-YUBIKEY-1QQQPQ\n").unwrap();

        assert_eq!(width(Lock::Keys, Some(&key), 8, 4), 4);
        assert_eq!(width(Lock::Keys, Some(&key), 8, 1), 1);
        assert_eq!(width(Lock::Keys, Some(&key), 1, 4), 1);
        assert_eq!(width(Lock::Passphrase, Some(&key), 8, 4), 1);
        assert_eq!(width(Lock::Keys, Some(&plugin), 8, 4), 1);
    }
}
