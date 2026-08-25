mod adopt;
mod automatic;
mod classes;
mod constants;
mod custom;
mod definitions;
mod editor;
mod host;
mod outputs;
mod paths;
mod repo;
mod run;
mod span;
mod units;
mod workspace;

pub use adopt::adoptable;
pub use automatic::{Automatic, automatic};
pub use classes::{ask, ask_missing};
pub use constants::SYSTEM_TEXT_MODE;
pub use custom::{customized, said};
pub use definitions::{offer_definitions, place_definitions, refresh_definitions};
pub use editor::{launch, open};
pub use host::host_name;
pub use outputs::{output_placement, output_relative, output_status, outputs};
pub use paths::{
    config_dir, data_dir, expand, home_dir, managed_relative, relative, repo_path, system_path,
};
pub use repo::{destination, managed_path, require_repo};
pub use run::{Run, dry_run, finished, set_command, set_dry_run, started};
pub use span::{seconds, span};
pub use units::{Managed, Unit, units, whole_link};
pub use workspace::{
    Workspace, configured, managed_entries, managed_files, managed_root, workspace,
};
