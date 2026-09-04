use crate::files::FileStatus;
use crate::output::Tone;

pub const DEFAULT_FILTER: &str = "warn";

pub const REFRESH_PANICKED: &str = "meta: refreshing the definitions panicked";

pub const GUARD_FAILED: &str = "git: leftover lock files will not be removed on interruption";

pub const VERBOSE_FILTER: &str = "luadot=debug";

pub const TRACE_FILTER: &str = "luadot=trace";

pub const UNSET: &str = "(none)";

pub const ADD_COMMAND: &str = "add";

pub const TAKE_COMMAND: &str = "take";

pub const PREVIEW_LIMIT: usize = 10;

pub const YES_FLAGS: &str = "-y or --yes";

pub const DIFF_PROGRAM: &str = "git";

pub const DIFF_ARGUMENTS: [&str; 1] = ["diff"];

pub const CUSTOM_ENTRY: &str = "entry";

pub const CUSTOM_RENDER: &str = "render";

pub const CUSTOM_SUMMARY: &str = "summary";

pub const MANAGED_FILES: &str = "managed";

pub const GENERATED_FILES: &str = "generated";

pub const TASK_UNKNOWN: &str = "is not a task the configuration registers";

pub const TASK_RUNS: &str =
    "(use \"luadot <name>\" to run one, \"luadot task --names\" to print the names alone)";

pub const DOC_PAGES: [(&str, &str, &str); 3] = [
    (
        "docs/ld.md",
        "## Every call",
        include_str!("../../docs/ld.md"),
    ),
    (
        "docs/templates.md",
        "## The resolver",
        include_str!("../../docs/templates.md"),
    ),
    (
        "docs/secrets.md",
        "## The calls",
        include_str!("../../docs/secrets.md"),
    ),
];

pub const DOC_HEADING: &str = "## ";

pub const DOC_ROOT: &str = "ld";

pub const DOC_API: &str = "ld.";

pub const DOC_ROW: &str = "| `ld.";

pub const DOC_REGISTERED_ROW: &str = "| `";

pub const DOC_CELLS: usize = 3;

pub const DOC_NO_ARGUMENTS: &str = "none";

pub const DOC_TAKES: &str = "takes ";

pub const DOC_WRITTEN_IN: &str = "written in ";

pub const DOC_DESCRIBES: &str =
    "(use \"luadot doc <call>\" to describe one, \"luadot doc ld\" to describe every one)";

pub const MAN_TITLE: &str = "LUADOT";

pub const MAN_MANUAL: &str = "User Commands";

pub const MAN_EMPTY_DATE: &str = " ";

pub const MAN_COMMAND_VALUE_NAME: &str = "COMMAND";

pub const MAN_HIDDEN_ARGS: [&str; 2] = ["help", "verbose"];

pub const MAN_SYNOPSIS_SECTION: &str = "SYNOPSIS";

pub const MAN_OPTIONS_SECTION: &str = "OPTIONS";

pub const MAN_COMMANDS_SECTION: &str = "COMMANDS";

pub const MAN_FILES_SECTION: &str = "FILES";

pub const MAN_ENVIRONMENT_SECTION: &str = "ENVIRONMENT";

pub const MAN_EXAMPLES_SECTION: &str = "EXAMPLES";

pub const MAN_SEE_ALSO_SECTION: &str = "SEE ALSO";

pub const MAN_FILES: [(&str, &str); 5] = [
    (
        "~/.config/luadot/config.lua",
        "The configuration, read before every command.",
    ),
    (
        "~/.local/share/luadot/repo",
        "The managed repository, when init or clone was left to pick the place.",
    ),
    (
        "~/.local/share/luadot/state.json",
        "The repository luadot manages and the answers this machine gave to the classes.",
    ),
    (
        "~/.local/share/luadot/backups",
        "One directory per run, holding what apply, tmpl alt and rm wrote over.",
    ),
    (
        "<repository>/.config/luadot/bootstrap.lua",
        "The script luadot bootstrap runs.",
    ),
];

pub const MAN_ENVIRONMENT: [(&str, &str); 5] = [
    (
        "XDG_CONFIG_HOME",
        "Where the configuration directory is looked for. ~/.config without it.",
    ),
    (
        "XDG_DATA_HOME",
        "Where the repository, the state and the backups are kept. ~/.local/share without it.",
    ),
    (
        "VISUAL, EDITOR",
        "The editor luadot edit and luadot config edit open. VISUAL wins, and vi is the fallback.",
    ),
    ("SHELL", "The shell luadot cd starts. /bin/sh without it."),
    (
        "RUST_LOG",
        "The log filter luadot reads when -v is not given.",
    ),
];

pub const MAN_EXAMPLES: [(&str, &str); 6] = [
    (
        "luadot init ~/dotfiles",
        "Create an empty repository and manage it.",
    ),
    (
        "luadot clone git@github.com:me/dotfiles.git",
        "Take over a repository another machine already filled.",
    ),
    (
        "luadot add ~/.zshrc",
        "Mirror a file into the repository and link it back.",
    ),
    (
        "luadot status",
        "List the managed files whose system copy drifted.",
    ),
    ("luadot apply", "Put every managed file back on the system."),
    (
        "luadot sync -m 'from the laptop'",
        "Commit what changed in the repository and push it.",
    ),
];

pub const MAN_SEE_ALSO: &str = "git(1), age(1), gpg(1)";

pub const MAN_DOCUMENTATION: &str = concat!(
    "The full documentation lives at ",
    env!("CARGO_PKG_REPOSITORY")
);

pub const ROFF_PREAMBLE: &str = r#".ie \n(.g .ds Aq \(aq
.el .ds Aq '
"#;

pub const STATUS_LABELS: [(FileStatus, &str, Tone); 5] = [
    (FileStatus::Synced, "synced", Tone::Good),
    (FileStatus::Missing, "missing", Tone::Warning),
    (FileStatus::Unlinked, "unlinked", Tone::Warning),
    (FileStatus::Differs, "differs", Tone::Bad),
    (FileStatus::Unreadable, "unreadable", Tone::Warning),
];

pub const STATUS_HEAD: &str = "On repository";

pub const STATUS_GENERATED_HEAD: &str = "Generated from templates";

pub const STATUS_CLEAN: &str = "nothing to apply, every managed file is synced";

pub const STATUS_GENERATED_CLEAN: &str = "nothing to apply, every generated file is synced";

pub const STATUS_SECTIONS: [(FileStatus, &str, &[&str]); 4] = [
    (
        FileStatus::Missing,
        "Files not on the system:",
        &["(use \"luadot apply <path>...\" to write them)"],
    ),
    (
        FileStatus::Unlinked,
        "Files not linked:",
        &["(use \"luadot relink\" to link them again)"],
    ),
    (
        FileStatus::Differs,
        "Files that differ:",
        &[
            "(use \"luadot diff <path>...\" to see what changed)",
            "(use \"luadot apply\" to keep the repository's copy, \"luadot take\" to keep the system's)",
        ],
    ),
    (
        FileStatus::Unreadable,
        "Files luadot could not decrypt:",
        &["(use \"luadot apply <path>...\" to see what the backend says)"],
    ),
];
