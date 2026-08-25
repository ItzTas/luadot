mod constants;
#[cfg(feature = "meta")]
mod describe;
mod gpu;
mod has_battery;
mod host;
mod ram;
mod table;

pub use constants::NAMESPACE;
#[cfg(feature = "meta")]
pub use describe::describe;
pub use table::table;
