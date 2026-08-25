mod constants;
mod install;
mod preload;

#[cfg(feature = "meta")]
pub use constants::{LPEG_MODULE, RE_MODULE};
pub use install::install;
pub use preload::preload;
