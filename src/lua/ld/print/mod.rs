mod constants;
#[cfg(feature = "meta")]
mod describe;
mod entry;
mod error;
mod field;
mod line;
mod note;
mod parse;
mod section;
mod table;
mod warn;

pub use constants::NAMESPACE;
#[cfg(feature = "meta")]
pub use describe::describe;
pub use table::table;
