mod link;
mod sync;

pub use link::{LinkMode, link};
pub use sync::{ConflictPolicy, SyncOutcome, sync_file};
