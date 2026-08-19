use super::super::table::Setter;
use super::{
    autocommit, autopush, backup, backup_age, backup_dir, backup_keep, conflict, link,
    passphrase_warn, pkg_warn, repo_dir,
};

pub const NAMESPACE: &str = "opt";

pub const AUTOCOMMIT: &str = "autocommit";

pub const AUTOPUSH: &str = "autopush";

pub const BACKUP: &str = "backup";

pub const BACKUP_AGE: &str = "backup_age";

pub const SPAN_KIND: &str = "a span like \"30d\"";

pub const BACKUP_DIR: &str = "backup_dir";

pub const BACKUP_KEEP: &str = "backup_keep";

pub const CONFLICT: &str = "conflict";

pub const LINK: &str = "link";

pub const PASSPHRASE_WARN: &str = "passphrase_warn";

pub const PKG_WARN: &str = "pkg_warn";

pub const REPO_DIR: &str = "repo_dir";

pub const SETTERS: [(&str, Setter); 11] = [
    (AUTOCOMMIT, autocommit::set),
    (AUTOPUSH, autopush::set),
    (BACKUP, backup::set),
    (BACKUP_AGE, backup_age::set),
    (BACKUP_DIR, backup_dir::set),
    (BACKUP_KEEP, backup_keep::set),
    (CONFLICT, conflict::set),
    (LINK, link::set),
    (PASSPHRASE_WARN, passphrase_warn::set),
    (PKG_WARN, pkg_warn::set),
    (REPO_DIR, repo_dir::set),
];
