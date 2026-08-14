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
