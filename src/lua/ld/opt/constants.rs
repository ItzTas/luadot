use super::super::table::Setter;
use super::set;

pub const NAMESPACE: &str = "opt";

pub const AUTOCOMMIT: &str = "autocommit";

pub const AUTOPUSH: &str = "autopush";

pub const BACKUP: &str = "backup";

pub const BACKUP_AGE: &str = "backup_age";

pub const SPAN_KIND: &str = "a span like \"30d\"";

pub const DIRECTORY_KIND: &str = "a directory";

pub const BACKUP_DIR: &str = "backup_dir";

pub const BACKUP_KEEP: &str = "backup_keep";

pub const CONFLICT: &str = "conflict";

pub const HINTS: &str = "hints";

pub const LFS: &str = "lfs";

pub const LINK: &str = "link";

pub const PASSPHRASE_WARN: &str = "passphrase_warn";

pub const PKG_WARN: &str = "pkg_warn";

pub const REPO_DIR: &str = "repo_dir";

pub const SETTERS: [(&str, Setter); 13] = [
    (AUTOCOMMIT, set::autocommit),
    (AUTOPUSH, set::autopush),
    (BACKUP, set::backup),
    (BACKUP_AGE, set::backup_age),
    (BACKUP_DIR, set::backup_dir),
    (BACKUP_KEEP, set::backup_keep),
    (CONFLICT, set::conflict),
    (HINTS, set::hints),
    (LFS, set::lfs),
    (LINK, set::link),
    (PASSPHRASE_WARN, set::passphrase_warn),
    (PKG_WARN, set::pkg_warn),
    (REPO_DIR, set::repo_dir),
];
