use super::table::Setter;
use super::{backend, identity, recipients};

pub const NAMESPACE: &str = "crypt";

pub const BACKEND: &str = "backend";

pub const IDENTITY: &str = "identity";

pub const RECIPIENTS: &str = "recipients";

pub const SETTERS: [(&str, Setter); 3] = [
    (BACKEND, backend::set),
    (IDENTITY, identity::set),
    (RECIPIENTS, recipients::set),
];
