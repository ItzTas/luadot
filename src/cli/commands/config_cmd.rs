use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use crate::lua::{self, Config, Rule};
use crate::output;
use crate::state;
use crate::utils;

const UNSET: &str = "(none)";

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
    #[command(about = "Open the configuration file in $VISUAL/$EDITOR")]
    Edit,
}

pub fn config_cmd(args: ConfigArgs) -> Result<()> {
    match args.action.unwrap_or(ConfigAction::Show) {
        ConfigAction::Show => show(),
        ConfigAction::Path => path(),
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

    output::field("repository", repo);
    output::field("config", file.display());
    output::field("link", config.link().name());
    output::field("conflict", config.conflict().name());
    output::field("backup", config.backup());

    print_ignore(&config);
    print_rules(&config);

    Ok(())
}

fn print_ignore(config: &Config) {
    if config.ignore().is_empty() {
        return;
    }

    output::section("ignore");
    for pattern in config.ignore() {
        output::line(format!("  {pattern}"));
    }
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

    if parts.is_empty() {
        return String::new();
    }

    format!("  {}", parts.join(" "))
}

fn path() -> Result<()> {
    output::line(lua::config_path()?.display());
    Ok(())
}

fn edit() -> Result<()> {
    let file = lua::config_path()?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("config: failed to create {}", parent.display()))?;
    }

    utils::open("config", &file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overrides_renders_only_the_keys_a_rule_carries() {
        let pattern = glob::Pattern::new(".ssh/**").unwrap();

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
