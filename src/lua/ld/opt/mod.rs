mod constants;
#[cfg(feature = "meta")]
mod describe;
mod set;
mod table;

pub use constants::{NAMESPACE, PKG_WARN};
#[cfg(feature = "meta")]
pub use describe::describe;
pub use table::table;
