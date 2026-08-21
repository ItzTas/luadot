use std::io::Write;

use anyhow::{Context, Result};
use clap::{Arg, Command, CommandFactory};
use clap_mangen::Man;
use clap_mangen::roff::{Inline, Roff, bold, italic, roman};

use crate::cli::Cli;

use super::super::constants::{
    MAN_COMMAND_VALUE_NAME, MAN_COMMANDS_SECTION, MAN_DOCUMENTATION, MAN_EMPTY_DATE,
    MAN_ENVIRONMENT, MAN_ENVIRONMENT_SECTION, MAN_EXAMPLES, MAN_EXAMPLES_SECTION, MAN_FILES,
    MAN_FILES_SECTION, MAN_HIDDEN_ARGS, MAN_MANUAL, MAN_OPTIONS_SECTION, MAN_SEE_ALSO,
    MAN_SEE_ALSO_SECTION, MAN_SYNOPSIS_SECTION, MAN_TITLE, ROFF_PREAMBLE,
};

pub fn man_cmd() -> Result<()> {
    print!("{}", page()?);
    Ok(())
}

fn page() -> Result<String> {
    let mut command = Cli::command().disable_help_subcommand(true);
    command.build();

    let man = Man::new(root(&command))
        .title(MAN_TITLE)
        .date(MAN_EMPTY_DATE)
        .manual(MAN_MANUAL);

    let mut page = String::from(ROFF_PREAMBLE);
    page.push_str(&rendered(|w| man.render_title(w))?);
    page.push_str(&rendered(|w| man.render_name_section(w))?);
    page.push_str(&rendered(|w| man.render_synopsis_section(w))?);
    page.push_str(&rendered(|w| man.render_description_section(w))?);
    page.push_str(&rendered(|w| man.render_options_section(w))?);
    page.push_str(&commands(&command)?);
    page.push_str(&extra());

    Ok(page)
}

fn root(command: &Command) -> Command {
    command
        .clone()
        .bin_name(command.get_name().to_string())
        .subcommand_value_name(MAN_COMMAND_VALUE_NAME)
}

fn entry(path: &str, sub: &Command) -> Command {
    sub.clone()
        .bin_name(path.to_string())
        .subcommand_value_name(MAN_COMMAND_VALUE_NAME)
        .mut_args(hide_inherited)
}

fn hide_inherited(arg: Arg) -> Arg {
    if MAN_HIDDEN_ARGS.contains(&arg.get_id().as_str()) {
        return arg.hide(true);
    }

    arg
}

fn commands(command: &Command) -> Result<String> {
    let name = command.get_name().to_string();

    let mut section = heading("SH", MAN_COMMANDS_SECTION);
    push_entries(&mut section, &name, command)?;

    Ok(section)
}

fn push_entries(section: &mut String, path: &str, command: &Command) -> Result<()> {
    for sub in command.get_subcommands().filter(|sub| !sub.is_hide_set()) {
        let path = format!("{path} {}", sub.get_name());
        section.push_str(&entry_section(&path, sub)?);
        push_entries(section, &path, sub)?;
    }

    Ok(())
}

fn entry_section(path: &str, sub: &Command) -> Result<String> {
    let man = Man::new(entry(path, sub));

    let synopsis = rendered(|w| man.render_synopsis_section(w))?;
    let options = rendered(|w| man.render_options_section(w))?;

    let mut section = heading("SS", path);
    section.push_str(&body(&synopsis, MAN_SYNOPSIS_SECTION)?);
    section.push_str(&about(sub));
    section.push_str(&body(&options, MAN_OPTIONS_SECTION)?);

    Ok(section)
}

fn about(sub: &Command) -> String {
    let Some(about) = sub.get_about() else {
        return String::new();
    };

    let mut roff = Roff::default();
    roff.control("PP", []);
    roff.text([roman(about.to_string())]);

    strip_preamble(&roff.render()).to_string()
}

fn extra() -> String {
    let mut roff = Roff::default();
    definitions(&mut roff, MAN_FILES_SECTION, &MAN_FILES, |term| {
        italic(term)
    });
    definitions(
        &mut roff,
        MAN_ENVIRONMENT_SECTION,
        &MAN_ENVIRONMENT,
        |term| bold(term),
    );
    definitions(&mut roff, MAN_EXAMPLES_SECTION, &MAN_EXAMPLES, |term| {
        bold(term)
    });

    roff.control("SH", [MAN_SEE_ALSO_SECTION]);
    roff.text([roman(MAN_SEE_ALSO)]);
    roff.control("PP", []);
    roff.text([roman(MAN_DOCUMENTATION)]);

    strip_preamble(&roff.render()).to_string()
}

fn definitions(
    roff: &mut Roff,
    section: &str,
    entries: &[(&str, &str)],
    term: impl Fn(&str) -> Inline,
) {
    roff.control("SH", [section]);

    for (name, description) in entries {
        roff.control("TP", []);
        roff.text([term(name)]);
        roff.text([roman(*description)]);
    }
}

fn heading(macro_name: &str, name: &str) -> String {
    let mut roff = Roff::default();
    roff.control(macro_name, [name]);

    strip_preamble(&roff.render()).to_string()
}

fn rendered(render: impl FnOnce(&mut dyn Write) -> std::io::Result<()>) -> Result<String> {
    let mut out = Vec::new();
    render(&mut out).context("man: failed to render the page")?;

    let text = String::from_utf8(out).context("man: the rendered page is not valid UTF-8")?;

    Ok(strip_preamble(&text).to_string())
}

fn strip_preamble(text: &str) -> &str {
    text.strip_prefix(ROFF_PREAMBLE).unwrap_or(text)
}

fn body(section: &str, name: &str) -> Result<String> {
    if section.trim().is_empty() {
        return Ok(String::new());
    }

    Ok(section
        .strip_prefix(&format!(".SH {name}\n"))
        .with_context(|| format!("man: the generated {name} section keeps its own heading"))?
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_reaches_the_page_as_a_subsection() {
        let page = page().unwrap();

        assert!(page.contains(".SS \"luadot add\""));
        assert!(page.contains(".SS \"luadot tmpl alt\""));
        assert!(page.contains(".SS \"luadot class unset\""));
    }

    #[test]
    fn the_apostrophe_preamble_is_written_once() {
        let page = page().unwrap();

        assert_eq!(page.matches(ROFF_PREAMBLE).count(), 1);
    }

    #[test]
    fn the_commands_carry_their_arguments_without_a_heading_of_their_own() {
        let page = page().unwrap();

        assert_eq!(page.matches(".SH SYNOPSIS").count(), 1);
        assert_eq!(page.matches(".SH OPTIONS").count(), 1);
        assert!(page.contains("Report what would be put back, writing nothing"));
    }

    #[test]
    fn the_inherited_flags_are_described_once() {
        let page = page().unwrap();

        assert_eq!(page.matches("Log what luadot is doing").count(), 1);
    }
}
