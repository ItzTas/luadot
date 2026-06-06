use std::collections::HashMap;

use anyhow::Result;

pub type Handler = fn(&[String]) -> Result<()>;

pub enum Command {
    Run(Handler),
    #[allow(dead_code)]
    Group(HashMap<&'static str, Command>),
}
