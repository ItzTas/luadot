#[cfg(feature = "meta")]
use super::super::super::signature::{Field, Kind};

pub const NAMESPACE: &str = "host";

pub const NAME: &str = "name";

pub const OS: &str = "os";

pub const ARCH: &str = "arch";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.sys.host";

#[cfg(feature = "meta")]
pub const DOC: &str = "The host.";

#[cfg(feature = "meta")]
pub const FIELDS: [Field; 3] = [
    Field {
        name: NAME,
        kind: Kind::String,
        doc: "The hostname.",
    },
    Field {
        name: OS,
        kind: Kind::String,
        doc: "The operating system, as Rust names it: `linux`.",
    },
    Field {
        name: ARCH,
        kind: Kind::String,
        doc: "The architecture: `x86_64`, `aarch64`.",
    },
];
