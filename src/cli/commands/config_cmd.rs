use anyhow::Result;
use clap::{Args, Subcommand};

use crate::lua::{self, Config, Rule};
use crate::output::{self, Message, Tone};
use crate::state;
use crate::utils;

use super::super::constants::UNSET;

const RULE_INDENT: usize = 2;

const PART_INDENT: usize = 4;

const RULE_COLUMN: usize = 44;

struct Row {
    head: String,
    parts: Vec<String>,
    overrides: String,
}

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

    let rows: Vec<Row> = config.rules().iter().map(Row::new).collect();
    let width = column_width(&rows);

    output::section("rules");
    for row in &rows {
        print_row(row, width);
    }
}

fn print_row(row: &Row, width: usize) {
    let column = match row.overrides.is_empty() {
        true => None,
        false => Some(width),
    };

    output::say(
        &Message::new(&row.head)
            .with_tail(&row.overrides)
            .with_look(Tone::Strong.into())
            .with_indent(RULE_INDENT)
            .with_column(column),
    );

    for part in &row.parts {
        output::say(
            &Message::new(part)
                .with_look(Tone::Muted.into())
                .with_indent(PART_INDENT),
        );
    }
}

fn column_width(rows: &[Row]) -> usize {
    let widest = rows
        .iter()
        .filter(|row| !row.overrides.is_empty())
        .map(|row| row.head.chars().count())
        .max()
        .unwrap_or(0);

    (widest + output::GAP.len()).min(RULE_COLUMN)
}

impl Row {
    fn new(rule: &Rule) -> Self {
        let parts = rule.pattern().parts();
        let overrides = overrides(rule);

        if parts.len() < 2 {
            return Self {
                head: rule.pattern().to_string(),
                parts: Vec::new(),
                overrides,
            };
        }

        Self {
            head: format!("any of {}", parts.len()),
            parts,
            overrides,
        }
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

    parts.join(" ")
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
            "link=symbolic"
        );
        assert_eq!(
            overrides(&Rule::new(
                pattern,
                Some(crate::files::LinkMode::Hard),
                Some(crate::files::ConflictPolicy::Skip)
            )),
            "link=hard conflict=skip"
        );
    }

    #[test]
    fn a_row_lists_the_alternatives_under_a_count() {
        let row = Row::new(&Rule::new(
            Matcher::Any(vec![glob(".cache/**"), glob("**/*.tmp")]),
            None,
            None,
        ));

        assert_eq!(row.head, "any of 2");
        assert_eq!(row.parts, vec![".cache/**", "**/*.tmp"]);
    }

    #[test]
    fn a_row_keeps_a_lone_pattern_on_its_line() {
        let row = Row::new(&Rule::new(glob(".ssh/**"), None, None));

        assert_eq!(row.head, ".ssh/**");
        assert!(row.parts.is_empty());
    }

    #[test]
    fn the_column_follows_the_widest_head_carrying_overrides() {
        let rows = [
            Row::new(&Rule::new(
                glob(".a-very-long-pattern-with-no-keys/**"),
                None,
                None,
            )),
            Row::new(&Rule::new(
                glob(".ssh/**"),
                Some(crate::files::LinkMode::Copy),
                None,
            )),
        ];

        assert_eq!(column_width(&rows), ".ssh/**".len() + output::GAP.len());
    }
}
