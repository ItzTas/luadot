use clap::{ArgAction, Parser, Subcommand};

use super::commands::{
    AddArgs, ApplyArgs, ClassArgs, CloneArgs, CompletionsArgs, ConfigArgs, DiffArgs, DocArgs,
    EditArgs, ExecArgs, GitArgs, InitArgs, MetaArgs, PushArgs, RekeyArgs, RestoreArgs, RmArgs,
    SetupArgs, StatusArgs, SyncArgs, TaskArgs, TmplArgs,
};

#[derive(Debug, Parser)]
#[command(
    name = "luadot",
    version,
    about = "A dotfiles manager configured in Lua",
    long_about = "luadot keeps your dotfiles in a git repository and puts them back on every \
machine you clone it to. The configuration is a Lua script instead of a static file, so one \
repository answers for a laptop, a desktop and a server without a branch or a copy per machine.\n\
\n\
The repository mirrors your home directory, path for path. Rules decide how each file is placed: \
linked hard, symbolic or copied, ignored, encrypted, or generated per machine by a template.",
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
    #[command(about = "Create an empty dotfiles repository and make it the managed one")]
    Init(InitArgs),
    #[command(about = "Clone a dotfiles repository and make it the managed one")]
    Clone(CloneArgs),
    #[command(about = "Start managing files or directories, linking them into the repository")]
    Add(AddArgs),
    #[command(about = "Stop managing files or directories, leaving your home copy in place")]
    Rm(RmArgs),
    #[command(about = "List the managed files whose system copy is not in sync")]
    Status(StatusArgs),
    #[command(about = "Show what the repository holds and the system does not")]
    Diff(DiffArgs),
    #[command(about = "Put the repository's files back on the system")]
    Apply(ApplyArgs),
    #[command(about = "Create the repository's templates and run them")]
    Tmpl(TmplArgs),
    #[command(about = "Put back the files an earlier apply or tmpl alt replaced")]
    Restore(RestoreArgs),
    #[command(about = "Open the repository's copy of a file in $VISUAL/$EDITOR")]
    Edit(EditArgs),
    #[command(about = "Re-encrypt the repository's secrets for the recipients set now")]
    Rekey(RekeyArgs),
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
    #[command(about = "Run a task the configuration registers; `luadot <name>` is the same")]
    Task(TaskArgs),
    #[command(about = "Start a shell in the repository")]
    Cd,
    #[command(about = "Stage what changed in the repository, commit it and push it")]
    Sync(SyncArgs),
    #[command(about = "Run git inside the repository")]
    Git(GitArgs),
    #[command(about = "Shorthand for `luadot git push`")]
    Push(PushArgs),
    #[command(about = "Describe the calls the configuration and the scripts have")]
    Doc(DocArgs),
    #[command(
        about = "Print the editor definitions of ld, or write them where the configuration is edited"
    )]
    Meta(MetaArgs),
    #[command(about = "Print a completion script for a shell")]
    Completions(CompletionsArgs),
    #[command(about = "Print the manual page")]
    Man,
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;
    use crate::lua::BUILTINS;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(args)
    }

    #[test]
    fn the_declaration_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn a_name_that_is_no_command_reaches_the_tasks_with_what_follows_it() {
        let cli = parse(&["luadot", "plug", "sync", "--all"]).unwrap();

        match cli.command {
            Cmd::External(words) => assert_eq!(words, ["plug", "sync", "--all"]),
            other => panic!("parsed {other:?}"),
        }
    }

    #[test]
    fn the_names_a_task_cannot_take_are_the_commands_declared() {
        let mut command = Cli::command();
        command.build();
        let mut declared: Vec<&str> = command
            .get_subcommands()
            .map(|sub| sub.get_name())
            .collect();
        let mut refused: Vec<&str> = BUILTINS.to_vec();
        declared.sort_unstable();
        refused.sort_unstable();

        assert_eq!(declared, refused);
    }

    #[test]
    fn git_keeps_every_argument_verbatim() {
        let cli = parse(&["luadot", "git", "commit", "-m", "msg"]).unwrap();

        match cli.command {
            Cmd::Git(args) => assert_eq!(args.args, ["commit", "-m", "msg"]),
            other => panic!("parsed {other:?}"),
        }
    }
}
