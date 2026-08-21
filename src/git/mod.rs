mod attributes;
mod auto;
mod clone;
mod commit;
mod constants;
mod empty;
#[cfg(test)]
pub mod fixture;
mod ignore;
mod init;
mod lfs;
mod locks;
mod push;
mod run;
mod scratch;
mod stage;

pub use attributes::{path as attributes_path, sync as sync_attributes};
pub use auto::auto;
pub use clone::{Cloned, clone};
pub use commit::{commit, committed, message, staged};
pub use ignore::{Excludes, Kind};
pub use init::init;
pub use lfs::{available as lfs_available, install as install_lfs};
pub use locks::guard as guard_locks;
pub use push::push;
pub use run::present;
pub use scratch::{record, scratch};
pub use stage::{stage, stage_all, unstage};
