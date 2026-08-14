use clap::{ArgAction, Parser, Subcommand};

use super::commands::{
    AddArgs, AltArgs, ApplyArgs, ClassArgs, CloneArgs, CompletionsArgs, ConfigArgs, EditArgs,
    ExecArgs, GitArgs, PushArgs, RestoreArgs, RmArgs, SetupArgs, StatusArgs,
};

#[derive(Debug, Parser)]
#[command(
    name = "luadot",
    version,
    about = "A dotfiles manager configured in Lua",
    arg_required_else_help = true
)]
pub struct Cli {
    #[arg(
        short,
        long,
        global = true,
        action = ArgAction::Count,
        help = "Log what luadot is doing (-vv logs more)"
    )]
    pub verbose: u8,
    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(about = "Clone a dotfiles repository and make it the managed one")]
    Clone(CloneArgs),
    #[command(about = "Start managing files or directories, linking them into the repository")]
    Add(AddArgs),
    #[command(about = "Stop managing files or directories, leaving your home copy in place")]
    Rm(RmArgs),
    #[command(about = "List the managed files whose system copy is not in sync")]
    Status(StatusArgs),
    #[command(about = "Put the repository's files back on the system")]
    Apply(ApplyArgs),
    #[command(about = "Run the templates and put the files they produce on the system")]
    Alt(AltArgs),
    #[command(about = "Put back the files an earlier apply or alt replaced")]
    Restore(RestoreArgs),
    #[command(about = "Open the repository's copy of a file in $VISUAL/$EDITOR")]
    Edit(EditArgs),
    #[command(about = "Run Lua with `ld` installed, from a string or a .lua file")]
    Exec(ExecArgs),
    #[command(about = "Show the resolved configuration, print its path, or open it")]
    Config(ConfigArgs),
    #[command(about = "List the declared classes and answer them for this machine")]
    Class(ClassArgs),
    #[command(about = "Run the repository's bootstrap.lua")]
    Bootstrap,
    #[command(about = "Run the repository's setup scripts")]
    Setup(SetupArgs),
    #[command(about = "Start a shell in the repository")]
    Cd,
    #[command(about = "Run git inside the repository")]
    Git(GitArgs),
    #[command(about = "Shorthand for `luadot git push`")]
    Push(PushArgs),
    #[command(about = "Print a completion script for a shell")]
    Completions(CompletionsArgs),
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use clap::error::ErrorKind;

    use super::super::commands::ClassAction;
    use super::*;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(args)
    }

    #[test]
    fn the_declaration_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn a_subcommand_reaches_its_arguments() {
        let cli = parse(&["luadot", "add", ".bashrc", ".vimrc"]).unwrap();

        match cli.command {
            Cmd::Add(args) => assert_eq!(args.paths, [".bashrc", ".vimrc"]),
            other => panic!("parsed {other:?}"),
        }
    }

    #[test]
    fn add_requires_a_path() {
        let err = parse(&["luadot", "add"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rm_takes_the_yes_flag() {
        let cli = parse(&["luadot", "rm", "-y", ".bashrc"]).unwrap();

        match cli.command {
            Cmd::Rm(args) => {
                assert!(args.yes);
                assert_eq!(args.paths, [".bashrc"]);
            }
            other => panic!("parsed {other:?}"),
        }
    }

    #[test]
    fn rm_rejects_an_unknown_option() {
        let err = parse(&["luadot", "rm", "--force", ".bashrc"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn git_keeps_every_argument_verbatim() {
        let cli = parse(&["luadot", "git", "commit", "-m", "msg"]).unwrap();

        match cli.command {
            Cmd::Git(args) => assert_eq!(args.args, ["commit", "-m", "msg"]),
            other => panic!("parsed {other:?}"),
        }
    }

    #[test]
    fn git_forwards_the_help_flag() {
        let cli = parse(&["luadot", "git", "--help"]).unwrap();

        match cli.command {
            Cmd::Git(args) => assert_eq!(args.args, ["--help"]),
            other => panic!("parsed {other:?}"),
        }
    }

    #[test]
    fn push_keeps_the_arguments_it_forwards() {
        let cli = parse(&["luadot", "push", "origin", "main"]).unwrap();

        match cli.command {
            Cmd::Push(args) => assert_eq!(args.args, ["origin", "main"]),
            other => panic!("parsed {other:?}"),
        }
    }

    #[test]
    fn exec_keeps_the_flags_after_the_target() {
        let cli = parse(&["luadot", "exec", "report.lua", "--json"]).unwrap();

        match cli.command {
            Cmd::Exec(args) => {
                assert_eq!(args.target, "report.lua");
                assert_eq!(args.args, ["--json"]);
            }
            other => panic!("parsed {other:?}"),
        }
    }

    #[test]
    fn bare_class_defaults_to_listing() {
        let cli = parse(&["luadot", "class"]).unwrap();

        match cli.command {
            Cmd::Class(args) => assert!(args.action.is_none()),
            other => panic!("parsed {other:?}"),
        }
    }

    #[test]
    fn class_set_takes_a_name_and_a_value() {
        let cli = parse(&["luadot", "class", "set", "email", "me@example.com"]).unwrap();

        match cli.command {
            Cmd::Class(ClassArgs {
                action: Some(ClassAction::Set { name, value }),
            }) => {
                assert_eq!(name.as_deref(), Some("email"));
                assert_eq!(value, ["me@example.com"]);
            }
            other => panic!("parsed {other:?}"),
        }
    }

    #[test]
    fn verbose_counts_from_any_position() {
        assert_eq!(parse(&["luadot", "-v", "status"]).unwrap().verbose, 1);
        assert_eq!(parse(&["luadot", "status", "-vv"]).unwrap().verbose, 2);
        assert_eq!(parse(&["luadot", "status"]).unwrap().verbose, 0);
    }

    #[test]
    fn version_prints_the_package_version() {
        let err = parse(&["luadot", "--version"]).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::DisplayVersion);
        assert!(err.to_string().contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn an_unknown_command_is_refused() {
        let err = parse(&["luadot", "nope"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn a_bare_invocation_shows_the_help() {
        let err = parse(&["luadot"]).unwrap_err();
        assert_eq!(
            err.kind(),
            ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
    }
}
