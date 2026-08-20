pub const AGE: &str = "age";

pub const GPG: &str = "gpg";

pub const GPG_FLAGS: [&str; 3] = ["--quiet", "--batch", "--yes"];

pub const GPG_PASSPHRASE_FLAGS: [&str; 2] = ["--quiet", "--yes"];

pub const PASSPHRASE_WARNING: &str = "passphrase mode is weaker than keys: one passphrase opens every secret, everyone sharing the repository shares it, and changing it means re-encrypting everything (silence this with `ld.opt.passphrase_warn(false)`)";

pub const SECRET_MODE: u32 = 0o600;

pub const WORKSPACE_PREFIX: &str = "luadot-edit";

pub const IDENTITY_FILE: &str = "identity";

pub const SHELL: &str = "sh";

pub const SHELL_ARG: &str = "-c";

pub const PLUGIN_BINARY: &str = "age-plugin-";

pub const PLUGIN_RECIPIENT: &str = "age1";

pub const PLUGIN_IDENTITY: &str = "AGE-PLUGIN-";

pub const EXECUTABLE: u32 = 0o111;
