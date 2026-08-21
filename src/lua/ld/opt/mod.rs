mod autocommit;
mod autopush;
mod backup;
mod backup_age;
mod backup_dir;
mod backup_keep;
mod conflict;
mod constants;
#[cfg(feature = "meta")]
mod describe;
mod link;
mod passphrase_warn;
mod pkg_warn;
mod repo_dir;
mod table;

pub use constants::{NAMESPACE, PKG_WARN};
#[cfg(feature = "meta")]
pub use describe::describe;
pub use table::table;
