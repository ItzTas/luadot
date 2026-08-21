mod ahead;
mod backend;
mod constants;
mod edit;
mod identity;
mod lock;
mod path;
mod plugin;
mod run;
mod secrets;
mod sync;

pub use ahead::Ahead;
pub use backend::Backend;
pub use edit::Workspace;
pub use identity::{Identity, Key, Provider};
pub use lock::Lock;
pub use path::{logical, split, stored, stored_variant};
pub use plugin::{
    for_identity as require_identity_plugins, for_recipients as require_recipient_plugins,
};
pub use run::{decrypt, decrypt_into, encrypt, encrypt_contents, require_recipients};
pub use secrets::Secrets;
pub use sync::{place, plain_status, status};
