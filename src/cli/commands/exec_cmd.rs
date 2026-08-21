use anyhow::Result;
use clap::Args;

use crate::lua;

#[derive(Debug, Args)]
pub struct ExecArgs {
    #[arg(value_name = "SOURCE", help = "Lua source, or a path to a .lua file")]
    pub target: String,
    #[arg(
        value_name = "ARGS",
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "Arguments the script reads through `ld.argv`"
    )]
    pub args: Vec<String>,
}

pub fn exec_cmd(args: ExecArgs) -> Result<()> {
    let config = lua::load_config()?;

    lua::run_exec("exec", &args.target, &config)
}
