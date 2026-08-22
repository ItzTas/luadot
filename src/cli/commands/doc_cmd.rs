use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Args;

use crate::lua;
use crate::output::{self, GAP};
use crate::utils;

use super::super::constants::{
    DOC_API, DOC_CELLS, DOC_DESCRIBES, DOC_HEADING, DOC_NO_ARGUMENTS, DOC_PAGES,
    DOC_REGISTERED_ROW, DOC_ROOT, DOC_ROW, DOC_TAKES, DOC_WRITTEN_IN,
};

#[derive(Debug, Args)]
pub struct DocArgs {
    #[arg(
        value_name = "CALL",
        help = "The call to describe, `ld.` optional; a namespace answers with everything under it, `ld` with every call, none names every one"
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
    pages: Vec<String>,
}

type Finder = fn(&Entry, &str, &str) -> bool;

pub fn doc_cmd(args: DocArgs) -> Result<()> {
    let entries = entries_with(&registered());

    if args.list {
        listing(&entries, false);
        return Ok(());
    }
    let Some(call) = args.call.as_deref() else {
        listing(&entries, true);
        return Ok(());
    };

    let found = found(&entries, call)?;
    describe(&found, found.len() == 1);

    Ok(())
}

fn listing(entries: &[Entry], describes: bool) {
    for name in listed(entries) {
        output::line(name);
    }

    if describes {
        output::hint(DOC_DESCRIBES);
    }
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

fn registered() -> Vec<(String, String)> {
    let pages = match registered_pages() {
        Ok(pages) => pages,
        Err(err) => {
            output::warn(format!("{err:#}"));
            return Vec::new();
        }
    };

    pages
        .into_iter()
        .filter_map(|path| match std::fs::read_to_string(&path) {
            Ok(text) => Some((path.display().to_string(), text)),
            Err(err) => {
                output::warn(format!("doc: failed to read {}: {err}", path.display()));
                None
            }
        })
        .collect()
}

fn registered_pages() -> Result<Vec<PathBuf>> {
    let config = lua::load_config()?;
    let pages = utils::configured("doc", &config)?.doc_pages().to_vec();

    Ok(pages)
}

fn entries_with(pages: &[(String, String)]) -> Vec<Entry> {
    let mut entries = entries();

    for (page, text) in pages {
        for line in text.lines() {
            if let Some(entry) = registered_row(page, line) {
                merge(&mut entries, entry);
            }
        }
    }

    entries
}

fn entries() -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();

    for (page, heading, text) in DOC_PAGES {
        for line in section(text, heading) {
            if let Some(entry) = row(page, line) {
                merge(&mut entries, entry);
            }
        }
    }

    entries
}

fn merge(entries: &mut Vec<Entry>, entry: Entry) {
    let Some(kept) = entries.iter_mut().find(|kept| kept.names == entry.names) else {
        entries.push(entry);
        return;
    };

    for page in entry.pages {
        if !kept.pages.contains(&page) {
            kept.pages.push(page);
        }
    }
}

fn section<'a>(text: &'a str, heading: &str) -> impl Iterator<Item = &'a str> {
    text.lines()
        .skip_while(move |line| line.trim_end() != heading)
        .skip(1)
        .take_while(|line| !line.starts_with(DOC_HEADING))
}

fn row(page: &str, line: &str) -> Option<Entry> {
    let line = line.trim();
    if !line.starts_with(DOC_ROW) {
        return None;
    }

    let cells = cells(line)?;
    entry(page, &cells, names(cells[0]))
}

fn registered_row(page: &str, line: &str) -> Option<Entry> {
    let line = line.trim();
    if !line.starts_with(DOC_REGISTERED_ROW) {
        return None;
    }

    let cells = cells(line)?;
    entry(page, &cells, registered_names(cells[0]))
}

fn cells(line: &str) -> Option<Vec<&str>> {
    let cells: Vec<&str> = line
        .trim_matches('|')
        .splitn(DOC_CELLS, '|')
        .map(str::trim)
        .collect();

    (cells.len() == DOC_CELLS).then_some(cells)
}

fn entry(page: &str, cells: &[&str], names: Vec<String>) -> Option<Entry> {
    if names.is_empty() {
        return None;
    }

    Some(Entry {
        signature: plain(cells[0]),
        arguments: plain(cells[1]),
        effect: plain(cells[2]),
        names,
        pages: vec![page.to_string()],
    })
}

fn plain(cell: &str) -> String {
    cell.replace('`', "")
}

fn names(cell: &str) -> Vec<String> {
    tokens(cell)
        .filter_map(|token| token.strip_prefix(DOC_API))
        .filter_map(|token| token.split('(').next())
        .map(str::to_string)
        .filter(|name| !name.is_empty())
        .collect()
}

fn registered_names(cell: &str) -> Vec<String> {
    tokens(cell)
        .map(|token| token.strip_prefix(DOC_API).unwrap_or(token))
        .filter_map(|token| token.split('(').next())
        .map(str::to_string)
        .filter(|name| name.contains('.'))
        .collect()
}

fn tokens(cell: &str) -> impl Iterator<Item = &str> {
    cell.split('`').skip(1).step_by(2)
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
    fn a_call_the_interface_does_not_carry_is_refused() {
        let error = found(&entries(), "opt.colour").unwrap_err().to_string();

        assert!(error.starts_with("doc: `opt.colour` is not a call"));
    }

    #[test]
    fn a_registered_page_answers_for_its_namespaced_calls_and_nothing_else() {
        let page = (
            "/plugins/lazyld/docs/lazyld.md".to_string(),
            "# lazyld\n\n| Call | Arguments | Effect |\n| --- | --- | --- |\n\
             | `lazyld.sync(names)` | plugin names | Clones what is missing. |\n\
             | `ld.lazyld.clean()` | none | Removes what is unused. |\n\n\
             | Key | Values | Effect |\n| --- | --- | --- |\n\
             | `branch` | a string | The branch to track. |\n"
                .to_string(),
        );

        let entries = entries_with(&[page]);

        let sync = only(&entries, "lazyld.sync");
        assert_eq!(sync.signature, "lazyld.sync(names)");
        assert_eq!(sync.effect, "Clones what is missing.");
        assert_eq!(sync.pages, ["/plugins/lazyld/docs/lazyld.md"]);
        assert_eq!(found(&entries, "lazyld").unwrap().len(), 2);
        assert!(found(&entries, "branch").is_err());
        assert_eq!(only(&entries, "opt.link").pages, ["docs/ld.md"]);
    }
}
