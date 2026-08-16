use super::table::Setter;
use super::{backup, backup_dir, backup_keep, link, pkg_warn, repo_dir};

pub const NAMESPACE: &str = "opt";

pub const BACKUP: &str = "backup";

pub const BACKUP_DIR: &str = "backup_dir";

pub const BACKUP_KEEP: &str = "backup_keep";

pub const LINK: &str = "link";

pub const PKG_WARN: &str = "pkg_warn";

pub const REPO_DIR: &str = "repo_dir";

pub const SETTERS: [(&str, Setter); 6] = [
    (BACKUP, backup::set),
    (BACKUP_DIR, backup_dir::set),
    (BACKUP_KEEP, backup_keep::set),
    (LINK, link::set),
    (PKG_WARN, pkg_warn::set),
    (REPO_DIR, repo_dir::set),
];
