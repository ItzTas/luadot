use anyhow::{Result, bail};
use clap::Args;

use crate::output::{self, GAP};

use super::super::constants::{
    DOC_API, DOC_CELLS, DOC_HEADING, DOC_NO_ARGUMENTS, DOC_PAGES, DOC_ROOT, DOC_ROW, DOC_TAKES,
    DOC_WRITTEN_IN,
};

#[derive(Debug, Args)]
pub struct DocArgs {
    #[arg(
        value_name = "CALL",
        required_unless_present = "list",
        help = "The call to describe, `ld.` optional; a namespace answers with everything under it, `ld` with every call"
    )]
    pub call: Option<String>,
    #[arg(short, long, help = "Print the name of every call, one per line")]
    pub list: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct Entry {
    signature: String,
    arguments: String,
    effect: String,
    names: Vec<String>,
    pages: Vec<&'static str>,
}

type Finder = fn(&Entry, &str, &str) -> bool;

pub fn doc_cmd(args: DocArgs) -> Result<()> {
    let entries = entries();

    if args.list {
        for name in listed(&entries) {
            output::line(name);
        }
        return Ok(());
    }

    let found = found(&entries, args.call.as_deref().unwrap_or_default())?;
    describe(&found, found.len() == 1);

    Ok(())
}

fn describe(entries: &[&Entry], pages: bool) {
    for entry in entries {
        output::section(&entry.signature);
        output::line(format!("{GAP}{}", entry.effect));

        if entry.arguments != DOC_NO_ARGUMENTS {
            output::hint(format!("{DOC_TAKES}{}", entry.arguments));
        }

        if pages {
            output::hint(format!("{DOC_WRITTEN_IN}{}", entry.pages.join(", ")));
        }
    }
}

fn found<'a>(entries: &'a [Entry], call: &str) -> Result<Vec<&'a Entry>> {
    let wanted = wanted(call);
    if wanted.is_empty() {
        return Ok(entries.iter().collect());
    }

    let namespace = format!("{wanted}.");
    for finder in [exact, under, near] as [Finder; 3] {
        let found: Vec<&Entry> = entries
            .iter()
            .filter(|entry| finder(entry, &wanted, &namespace))
            .collect();

        if !found.is_empty() {
            return Ok(found);
        }
    }

    bail!("doc: `{call}` is not a call of the interface, `luadot doc --list` names every one")
}

fn exact(entry: &Entry, wanted: &str, _: &str) -> bool {
    entry.names.iter().any(|name| name == wanted)
}

fn under(entry: &Entry, _: &str, namespace: &str) -> bool {
    entry.names.iter().any(|name| name.starts_with(namespace))
}

fn near(entry: &Entry, wanted: &str, _: &str) -> bool {
    entry.names.iter().any(|name| name.contains(wanted))
}

fn wanted(call: &str) -> String {
    let call = call.trim().trim_end_matches("()");
    let call = call.strip_prefix(DOC_API).unwrap_or(call);

    if call == DOC_ROOT {
        return String::new();
    }

    call.trim_matches('.').to_string()
}

fn listed(entries: &[Entry]) -> Vec<String> {
    let mut names: Vec<String> = entries
        .iter()
        .flat_map(|entry| entry.names.iter().cloned())
        .collect();

    names.sort();
    names.dedup();

    names
}

fn entries() -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();

    for (page, heading, text) in DOC_PAGES {
        for line in section(text, heading) {
            let Some(entry) = row(page, line) else {
                continue;
            };

            let Some(kept) = entries.iter_mut().find(|kept| kept.names == entry.names) else {
                entries.push(entry);
                continue;
            };

            if !kept.pages.contains(&page) {
                kept.pages.push(page);
            }
        }
    }

    entries
}

fn section<'a>(text: &'a str, heading: &str) -> impl Iterator<Item = &'a str> {
    text.lines()
        .skip_while(move |line| line.trim_end() != heading)
        .skip(1)
        .take_while(|line| !line.starts_with(DOC_HEADING))
}

fn row(page: &'static str, line: &str) -> Option<Entry> {
    let line = line.trim();
    if !line.starts_with(DOC_ROW) {
        return None;
    }

    let cells: Vec<&str> = line
        .trim_matches('|')
        .splitn(DOC_CELLS, '|')
        .map(str::trim)
        .collect();

    if cells.len() != DOC_CELLS {
        return None;
    }

    let names = names(cells[0]);
    if names.is_empty() {
        return None;
    }

    Some(Entry {
        signature: plain(cells[0]),
        arguments: plain(cells[1]),
        effect: plain(cells[2]),
        names,
        pages: vec![page],
    })
}

fn plain(cell: &str) -> String {
    cell.replace('`', "")
}

fn names(cell: &str) -> Vec<String> {
    cell.split('`')
        .skip(1)
        .step_by(2)
        .filter_map(|token| token.strip_prefix(DOC_API))
        .filter_map(|token| token.split('(').next())
        .map(str::to_string)
        .filter(|name| !name.is_empty())
        .collect()
}

#[cfg(test)]
pub fn documented(name: &str) -> bool {
    let entries = entries();
    let mut path = name;

    loop {
        if entries
            .iter()
            .any(|entry| entry.names.iter().any(|name| name == path))
        {
            return true;
        }

        let Some((parent, _)) = path.rsplit_once('.') else {
            return false;
        };

        path = parent;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn only<'a>(entries: &'a [Entry], call: &str) -> &'a Entry {
        let found = found(entries, call).unwrap();
        assert_eq!(found.len(), 1, "{call} answered with {}", found.len());

        found[0]
    }

    #[test]
    fn a_row_carries_its_signature_its_arguments_and_its_effect() {
        let entries = entries();
        let entry = only(&entries, "opt.link");

        assert_eq!(entry.signature, "ld.opt.link(mode)");
        assert_eq!(entry.arguments, r#""hard", "symbolic", "copy""#);
        assert_eq!(
            entry.effect,
            "Default strategy used to link a managed file."
        );
    }

    #[test]
    fn a_row_naming_several_calls_answers_for_each_of_them() {
        let entries = entries();

        assert_eq!(only(&entries, "print.note"), only(&entries, "print.error"));
    }

    #[test]
    fn a_call_written_twice_is_kept_once_and_names_both_pages() {
        let entries = entries();
        let entry = only(&entries, "crypt.lock");

        assert_eq!(entry.pages, ["docs/ld.md", "docs/secrets.md"]);
    }

    #[test]
    fn the_prefix_and_the_parentheses_are_optional() {
        let entries = entries();

        assert_eq!(only(&entries, "ld.rules()"), only(&entries, "rules"));
    }

    #[test]
    fn a_namespace_answers_with_every_call_under_it() {
        let entries = entries();
        let found = found(&entries, "regex").unwrap();

        assert_eq!(found.len(), 7);
        assert!(
            found
                .iter()
                .all(|entry| entry.signature.starts_with("ld.regex."))
        );
    }

    #[test]
    fn the_interface_itself_answers_with_every_call() {
        let entries = entries();

        assert_eq!(found(&entries, "ld").unwrap().len(), entries.len());
    }

    #[test]
    fn a_piece_of_a_name_answers_with_what_carries_it() {
        let entries = entries();
        let found = found(&entries, "backup").unwrap();

        assert_eq!(found.len(), 4);
    }

    #[test]
    fn a_call_the_interface_does_not_carry_is_refused() {
        let error = found(&entries(), "opt.colour").unwrap_err().to_string();

        assert!(error.starts_with("doc: `opt.colour` is not a call"));
    }

    #[test]
    fn the_names_are_listed_once_each_in_order() {
        let names = listed(&entries());

        assert!(names.contains(&"alt.render".to_string()));
        assert!(names.is_sorted());
        assert_eq!(names.iter().filter(|name| *name == "crypt.lock").count(), 1);
    }
}
