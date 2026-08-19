mod automatic;
mod classes;
mod constants;
mod custom;
mod editor;
mod host;
mod outputs;
mod paths;
mod repo;
mod run;
mod span;
mod workspace;

pub use automatic::{Automatic, automatic};
pub use classes::{ask, ask_missing};
pub use constants::SYSTEM_TEXT_MODE;
pub use custom::{customized, said};
pub use editor::{launch, open};
pub use host::host_name;
pub use outputs::{
    escalated_output_status, generated_mode, output_relative, output_status, outputs,
};
pub use paths::{
    config_dir, data_dir, expand, home_dir, is_managed, is_root, managed_relative, relative,
    repo_path, system_path,
};
pub use repo::{destination, managed_path, require_repo};
pub use run::Run;
pub use span::{seconds, span};
pub use workspace::{Workspace, managed_entries, managed_files, managed_root, workspace};
