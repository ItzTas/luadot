#[cfg(feature = "meta")]
use super::super::signature::{Field, Kind, Signature};
#[cfg(feature = "meta")]
use super::ram;

pub const NAMESPACE: &str = "sys";

pub const HAS_BATTERY: &str = "has_battery";

pub const POWER_SUPPLY_DIR: &str = "/sys/class/power_supply";

pub const TYPE_FILE: &str = "type";

pub const SCOPE_FILE: &str = "scope";

pub const BATTERY_TYPE: &str = "Battery";

pub const DEVICE_SCOPE: &str = "Device";

#[cfg(feature = "meta")]
pub const NAMESPACE_TYPENAME: &str = "ld.sys";

#[cfg(feature = "meta")]
pub const DOC: &str = "The machine the script is running on.";

#[cfg(feature = "meta")]
pub const FIELDS: [Field; 1] = [Field {
    name: ram::NAMESPACE,
    kind: Kind::Integer,
    doc: "The memory of the machine, in bytes, the kernel's raw `MemTotal`: a little under the installed memory, so round it yourself: `math.ceil(ld.sys.ram / 1024 ^ 3)`.",
}];

#[cfg(feature = "meta")]
pub const SIGNATURES: [Signature; 1] = [Signature {
    name: HAS_BATTERY,
    params: &[],
    returns: &[Kind::Boolean],
    doc: "`true` on a machine with a battery of its own, `false` on one without; the battery of a mouse or a keyboard does not count.",
}];
