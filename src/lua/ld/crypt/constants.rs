use super::super::table::Setter;
use super::{backend, lock};

pub const NAMESPACE: &str = "crypt";

pub const BACKEND: &str = "backend";

pub const LOCK: &str = "lock";

pub const LOCK_KEYS: &str = "crypt.lock";

pub const LOCK_KIND: &str =
    "\"passphrase\" or a table of `recipients`, `identity` and `identity_command`";

pub const LOCK_CONFLICT: &str = "`ld.crypt.lock` locks with a passphrase or with keys, never both; drop `passphrase` or drop the keys beside it";

pub const PASSPHRASE: &str = "passphrase";

pub const IDENTITY: &str = "identity";

pub const RECIPIENTS: &str = "recipients";

pub const SETTERS: [(&str, Setter); 2] = [(BACKEND, backend::set), (LOCK, lock::set)];

pub const TYPE: &str = "type";

pub const FILE: &str = "file";

pub const COMMAND: &str = "command";

pub const IDENTITY_KEYS: &str = "crypt.lock.identity";

pub const IDENTITY_TYPE: &str = "identity type";

pub const IDENTITY_KIND: &str =
    "a path, a command line, or a table carrying `type` and what it names";

pub const FILE_ALONE: &str = "`ld.crypt.lock`'s identity of type `file` takes one path";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    File,
    Command,
}

pub const IDENTITY_TYPES: [(&str, Kind); 2] = [(COMMAND, Kind::Command), (FILE, Kind::File)];
