mod captures;
mod constants;
#[cfg(feature = "meta")]
mod describe;
mod escape;
mod find;
#[cfg(test)]
mod fixture;
mod gmatch;
mod gsub;
mod r#match;
mod parse;
mod split;
mod table;
mod test;

pub use constants::NAMESPACE;
#[cfg(feature = "meta")]
pub use describe::describe;
pub use table::table;
