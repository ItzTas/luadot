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

pub const IDENTITY_COMMAND: &str = "identity_command";

pub const RECIPIENTS: &str = "recipients";

pub const SETTERS: [(&str, Setter); 2] = [(BACKEND, backend::set), (LOCK, lock::set)];
