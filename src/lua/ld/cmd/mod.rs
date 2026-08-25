mod constants;
#[cfg(feature = "meta")]
mod describe;
#[cfg(test)]
mod fixture;
mod program;
mod shell;
mod table;

pub use constants::NAMESPACE;
#[cfg(feature = "meta")]
pub use describe::describe;
pub use table::table;
