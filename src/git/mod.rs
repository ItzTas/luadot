mod auto;
mod clone;
mod commit;
mod constants;
mod empty;
mod ignore;
mod init;
mod push;
mod run;
mod stage;

pub use auto::auto;
pub use clone::{Cloned, clone};
pub use commit::{commit, committed, message, staged};
pub use ignore::{Excludes, Kind};
pub use init::init;
pub use push::push;
pub use run::present;
pub use stage::{stage, stage_all, unstage};
