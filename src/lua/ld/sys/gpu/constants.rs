#[cfg(feature = "meta")]
use super::super::super::constants::INTEGER_INDEX;
#[cfg(feature = "meta")]
use super::super::super::signature::{Field, Kind};

pub const NAMESPACE: &str = "gpu";

pub const VENDOR: &str = "vendor";

pub const NAME: &str = "name";

pub const DRIVER: &str = "driver";

pub const DRM_DIR: &str = "/sys/class/drm";

pub const CARD_PREFIX: &str = "card";

pub const DEVICE_LINK: &str = "device";

pub const VENDOR_FILE: &str = "device/vendor";

pub const DRIVER_LINK: &str = "device/driver";

pub const LSPCI: &str = "lspci";

pub const LSPCI_ARGS: [&str; 2] = ["-mm", "-D"];

pub const VENDORS: [(&str, &str); 9] = [
    ("0x10de", "nvidia"),
    ("0x1002", "amd"),
    ("0x1022", "amd"),
    ("0x8086", "intel"),
    ("0x1af4", "virtio"),
    ("0x1234", "qemu"),
    ("0x15ad", "vmware"),
    ("0x1414", "microsoft"),
    ("0x1a03", "aspeed"),
];

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.sys.gpu";

#[cfg(feature = "meta")]
pub const DOC: &str =
    "The first card, and every card as a list: `for _, card in ipairs(ld.sys.gpu)`.";

#[cfg(feature = "meta")]
pub const CARD_TYPENAME: &str = "ld.Card";

#[cfg(feature = "meta")]
pub const CARD_DOC: &str = "One graphics card.";

#[cfg(feature = "meta")]
pub const CARD_FIELDS: [Field; 3] = [
    Field {
        name: VENDOR,
        kind: Kind::String,
        doc: "A short name (`nvidia`, `amd`, `intel`), or the PCI identifier when the vendor is not a known one.",
    },
    Field {
        name: NAME,
        kind: Kind::String,
        doc: "The model as `lspci` reports it, empty when `lspci` is not installed.",
    },
    Field {
        name: DRIVER,
        kind: Kind::String,
        doc: "The kernel driver bound to the card.",
    },
];

#[cfg(feature = "meta")]
pub const LIST_FIELDS: [Field; 1] = [Field {
    name: INTEGER_INDEX,
    kind: Kind::Named(CARD_TYPENAME),
    doc: "Every card, in order.",
}];
