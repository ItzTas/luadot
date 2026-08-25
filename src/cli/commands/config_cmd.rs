use anyhow::Result;
use clap::{Args, Subcommand};

use crate::lua::{self, Config, Rule};
use crate::output;
use crate::state;
use crate::utils;

use super::super::constants::UNSET;

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: Option<ConfigAction>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    #[command(about = "Show the resolved configuration")]
    Show,
    #[command(about = "Print the path of the configuration file")]
    Path,
    #[command(about = "Print the path of the managed repository")]
    Repo,
    #[command(about = "Open the configuration file in $VISUAL/$EDITOR")]
    Edit,
}

pub fn config_cmd(args: ConfigArgs) -> Result<()> {
    match args.action.unwrap_or(ConfigAction::Show) {
        ConfigAction::Show => show(),
        ConfigAction::Path => path(),
        ConfigAction::Repo => repo(),
        ConfigAction::Edit => edit(),
    }
}

fn show() -> Result<()> {
    let state = state::load()?;
    let repo = state
        .repo()
        .map(|repo| repo.display().to_string())
        .unwrap_or_else(|| UNSET.to_string());

    let file = lua::config_path()?;
    let config = lua::load_config()?;
    let config = utils::configured("config", &config)?;

    output::field("repository", repo);
    output::field("config", file.display());
    output::field("link", config.link().name());
    output::field("conflict", config.conflict().name());
    output::field("backup", config.backup());

    print_rules(&config);

    Ok(())
}

fn print_rules(config: &Config) {
    if config.rules().is_empty() {
        return;
    }

    output::section("rules");
    for rule in config.rules() {
        output::line(format!("  {}{}", rule.pattern(), overrides(rule)));
    }
}

fn overrides(rule: &Rule) -> String {
    let mut parts = Vec::new();
    if let Some(link) = rule.link() {
        parts.push(format!("link={}", link.name()));
    }
    if let Some(conflict) = rule.conflict() {
        parts.push(format!("conflict={}", conflict.name()));
    }
    if let Some(on_change) = rule.on_change() {
        parts.push(format!("on_change=`{on_change}`"));
    }
    if let Some(track) = rule.track() {
        parts.push(format!("track={}", track.name()));
    }
    if let Some(whole) = rule.whole() {
        parts.push(format!("whole={whole}"));
    }

    if parts.is_empty() {
        return String::new();
    }

    format!("  {}", parts.join(" "))
}

fn path() -> Result<()> {
    output::line(lua::config_path()?.display());
    Ok(())
}

fn repo() -> Result<()> {
    let config = lua::load_config()?;
    let repo = utils::require_repo("config", utils::configured("config", &config)?.repo_dir())?;
    output::line(repo.display());
    Ok(())
}

fn edit() -> Result<()> {
    let file = lua::config_path()?;
    lua::place_starter("config", &file)?;

    utils::open("config", &file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::Matcher;

    fn glob(raw: &str) -> Matcher {
        Matcher::Glob(glob::Pattern::new(raw).unwrap())
    }

    #[test]
    fn overrides_renders_only_rule_keys() {
        let pattern = glob(".ssh/**");

        assert_eq!(overrides(&Rule::new(pattern.clone(), None, None)), "");
        assert_eq!(
            overrides(&Rule::new(
                pattern.clone(),
                Some(crate::files::LinkMode::Symbolic),
                None
            )),
            "  link=symbolic"
        );
        assert_eq!(
            overrides(&Rule::new(
                pattern,
                Some(crate::files::LinkMode::Hard),
                Some(crate::files::ConflictPolicy::Skip)
            )),
            "  link=hard conflict=skip"
        );
    }
}
