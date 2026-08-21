#[cfg(feature = "meta")]
use super::super::signature::{Kind, Param, Signature};

pub const NAMESPACE: &str = "pkg";

pub const INSTALL: &str = "install";

pub const SUDO: &str = "sudo";

pub const MANAGERS: [(&str, &[&str]); 3] = [
    ("pacman", &["-S", "--needed", "--noconfirm"]),
    ("apt-get", &["install", "-y"]),
    ("dnf", &["install", "-y"]),
];

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.pkg";

#[cfg(feature = "meta")]
pub const DOC: &str = "The system package manager: pacman, apt-get or dnf, whichever is on the `PATH`, through `sudo` when it is there.";

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 1] = [Signature {
    name: INSTALL,
    params: &[Param {
        name: "packages",
        kind: Kind::Or(&[Kind::String, Kind::List(&Kind::String)]),
    }],
    returns: &[],
    doc: "Installs packages through the system package manager. Slow: it belongs in `bootstrap.lua` or a setup script, and warns elsewhere.",
}];
