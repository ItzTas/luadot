mod constants;
mod fs;
mod link;
mod mirror;
mod predict;
mod status;
mod sync;
mod system;
mod template;
mod walk;
mod write;

pub use fs::{
    create_parent, exists, link_target, mode_bits, regular_file, remove_existing, write_mode,
};
pub use link::{LinkMode, link};
pub use mirror::{Mirror, Side, Tracked};
pub use predict::predict;
pub use status::{FileStatus, file_status};
pub use sync::{ConflictPolicy, SyncOutcome, refused, sync_file};
pub use system::{
    Staged, effective_mode, escalate_entry, escalated_read, escalated_status, import_system,
    inspect_system, permission_denied, place_contents, read_contents, stage_text, sync_system,
};
pub use template::{is_template, template_dir, template_target};
pub use walk::{Entry, collect_entries, collect_files};
pub use write::{text_status, write_file};
