mod atomic;
mod constants;
mod fs;
mod link;
mod mirror;
mod placement;
mod predict;
mod status;
mod sync;
mod template;
mod unit;
mod walk;
mod write;

pub use atomic::{replace_contents, replace_file};
pub use fs::{
    create_parent, effective_mode, exists, link_at, link_target, metadata, mode_bits, private_dir,
    prune_parents, read_contents, regular_file, write_mode,
};
pub use link::{LinkMode, link};
pub use mirror::{Mirror, Side, Tracked};
pub use placement::{Attributes, Placement};
pub use predict::predict;
pub use status::{FileStatus, file_status};
pub use sync::{ConflictPolicy, SyncOutcome, refused, sync_file};
pub use template::{is_template, template_dir, template_target};
pub use unit::{copy_tree, dir_status, remove_entry, sync_dir};
pub use walk::{Entry, collect_entries, collect_files};
pub use write::{text_status, write_file};
