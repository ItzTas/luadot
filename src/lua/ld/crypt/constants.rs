use super::super::table::Setter;
use super::{backend, identity, identity_command, passphrase, passphrase_warn, recipients};

pub const NAMESPACE: &str = "crypt";

pub const BACKEND: &str = "backend";

pub const IDENTITY: &str = "identity";

pub const IDENTITY_COMMAND: &str = "identity_command";

pub const PASSPHRASE: &str = "passphrase";

pub const PASSPHRASE_WARN: &str = "passphrase_warn";

pub const RECIPIENTS: &str = "recipients";

pub const SETTERS: [(&str, Setter); 6] = [
    (BACKEND, backend::set),
    (IDENTITY, identity::set),
    (IDENTITY_COMMAND, identity_command::set),
    (PASSPHRASE, passphrase::set),
    (PASSPHRASE_WARN, passphrase_warn::set),
    (RECIPIENTS, recipients::set),
];
