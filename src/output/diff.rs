use std::fmt::Display;

use super::message::Message;
use super::print::say;
use super::tone::Tone;

const CONTEXT: usize = 3;

const CELLS: usize = 4_000_000;

const INDENT: usize = 2;

const BINARY: &str = "binary content";

const SAME: &str = "no content change";

const NO_NEWLINE: &str = "\\ no newline at end of file";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Kept,
    Removed,
    Added,
}

const MARKS: [(Mark, char, Tone); 3] = [
    (Mark::Kept, ' ', Tone::Muted),
    (Mark::Removed, '-', Tone::Bad),
    (Mark::Added, '+', Tone::Good),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Edit<'a> {
    mark: Mark,
    text: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
struct Hunk<'a> {
    before: (usize, usize),
    after: (usize, usize),
    edits: Vec<Edit<'a>>,
}

pub fn changes(before: Option<&[u8]>, after: &[u8]) {
    let (Some(before), Some(after)) = (readable(before.unwrap_or_default()), readable(after))
    else {
        shade(Tone::Muted, BINARY);
        return;
    };

    let hunks = hunks(&lines(before), &lines(after));
    if hunks.is_empty() {
        shade(Tone::Muted, SAME);
        return;
    }

    for hunk in &hunks {
        show(hunk);
    }
}

fn readable(bytes: &[u8]) -> Option<&str> {
    if bytes.contains(&0) {
        return None;
    }

    std::str::from_utf8(bytes).ok()
}

fn lines(text: &str) -> Vec<&str> {
    text.split_inclusive('\n').collect()
}

fn show(hunk: &Hunk) {
    for (tone, text) in rows(hunk) {
        shade(tone, text);
    }
}

fn rows(hunk: &Hunk) -> Vec<(Tone, String)> {
    let header = format!("@@ -{} +{} @@", span(hunk.before), span(hunk.after));
    let mut rows = vec![(Tone::Muted, header)];

    for edit in &hunk.edits {
        let (sign, tone) = shown(edit.mark);
        rows.push((tone, format!("{sign} {}", stripped(edit.text))));

        if !edit.text.ends_with('\n') {
            rows.push((Tone::Muted, NO_NEWLINE.to_string()));
        }
    }

    rows
}

fn span((start, count): (usize, usize)) -> String {
    match count {
        0 => format!("{},0", start.saturating_sub(1)),
        _ => format!("{start},{count}"),
    }
}

fn shown(mark: Mark) -> (char, Tone) {
    MARKS
        .iter()
        .find(|(kind, _, _)| *kind == mark)
        .map(|(_, sign, tone)| (*sign, *tone))
        .unwrap_or((' ', Tone::Muted))
}

fn stripped(text: &str) -> &str {
    let text = text.strip_suffix('\n').unwrap_or(text);

    text.strip_suffix('\r').unwrap_or(text)
}

fn shade(tone: Tone, text: impl Display) {
    say(&Message::new(text)
        .with_look(tone.into())
        .with_indent(INDENT));
}

fn hunks<'a>(before: &[&'a str], after: &[&'a str]) -> Vec<Hunk<'a>> {
    let edits = edits(before, after);

    ranges(&edits)
        .into_iter()
        .map(|(start, end)| hunk(&edits, start, end))
        .collect()
}

fn edits<'a>(before: &[&'a str], after: &[&'a str]) -> Vec<Edit<'a>> {
    let head = prefix(before, after);
    let tail = suffix(&before[head..], &after[head..]);

    let mut edits: Vec<Edit<'a>> = before[..head].iter().map(kept).collect();
    edits.extend(middle(
        &before[head..before.len() - tail],
        &after[head..after.len() - tail],
    ));
    edits.extend(before[before.len() - tail..].iter().map(kept));

    edits
}

fn prefix(before: &[&str], after: &[&str]) -> usize {
    before
        .iter()
        .zip(after.iter())
        .take_while(|(left, right)| left == right)
        .count()
}

fn suffix(before: &[&str], after: &[&str]) -> usize {
    before
        .iter()
        .rev()
        .zip(after.iter().rev())
        .take_while(|(left, right)| left == right)
        .count()
}

fn middle<'a>(before: &[&'a str], after: &[&'a str]) -> Vec<Edit<'a>> {
    if before.is_empty() || after.is_empty() || before.len() * after.len() > CELLS {
        return before
            .iter()
            .map(removed)
            .chain(after.iter().map(added))
            .collect();
    }

    backtracked(&table(before, after), before, after)
}

fn table(before: &[&str], after: &[&str]) -> Vec<Vec<u32>> {
    let mut rows = vec![vec![0u32; after.len() + 1]; before.len() + 1];

    for (row, left) in before.iter().enumerate().rev() {
        for (column, right) in after.iter().enumerate().rev() {
            rows[row][column] = match left == right {
                true => rows[row + 1][column + 1] + 1,
                false => rows[row + 1][column].max(rows[row][column + 1]),
            };
        }
    }

    rows
}

fn backtracked<'a>(rows: &[Vec<u32>], before: &[&'a str], after: &[&'a str]) -> Vec<Edit<'a>> {
    let mut edits = Vec::new();
    let (mut row, mut column) = (0usize, 0usize);

    while row < before.len() && column < after.len() {
        if before[row] == after[column] {
            edits.push(kept(&before[row]));
            row += 1;
            column += 1;
            continue;
        }

        if rows[row + 1][column] >= rows[row][column + 1] {
            edits.push(removed(&before[row]));
            row += 1;
            continue;
        }

        edits.push(added(&after[column]));
        column += 1;
    }

    edits.extend(before[row..].iter().map(removed));
    edits.extend(after[column..].iter().map(added));

    edits
}

fn ranges(edits: &[Edit]) -> Vec<(usize, usize)> {
    let mut ranges: Vec<(usize, usize)> = Vec::new();

    for (index, edit) in edits.iter().enumerate() {
        if edit.mark == Mark::Kept {
            continue;
        }

        let start = index.saturating_sub(CONTEXT);
        let end = (index + CONTEXT + 1).min(edits.len());
        match ranges.last_mut() {
            Some(last) if last.1 >= start => last.1 = end,
            _ => ranges.push((start, end)),
        }
    }

    ranges
}

fn hunk<'a>(edits: &[Edit<'a>], start: usize, end: usize) -> Hunk<'a> {
    let span = &edits[start..end];

    Hunk {
        before: (
            counted(&edits[..start], Mark::Added) + 1,
            counted(span, Mark::Added),
        ),
        after: (
            counted(&edits[..start], Mark::Removed) + 1,
            counted(span, Mark::Removed),
        ),
        edits: span.to_vec(),
    }
}

fn counted(edits: &[Edit], absent: Mark) -> usize {
    edits.iter().filter(|edit| edit.mark != absent).count()
}

fn kept<'a>(text: &&'a str) -> Edit<'a> {
    Edit {
        mark: Mark::Kept,
        text,
    }
}

fn removed<'a>(text: &&'a str) -> Edit<'a> {
    Edit {
        mark: Mark::Removed,
        text,
    }
}

fn added<'a>(text: &&'a str) -> Edit<'a> {
    Edit {
        mark: Mark::Added,
        text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered(before: &str, after: &str) -> Vec<String> {
        hunks(&lines(before), &lines(after))
            .iter()
            .flat_map(|hunk| rows(hunk).into_iter().map(|(_, text)| text))
            .collect()
    }

    #[test]
    fn a_changed_line_carries_its_context() {
        let before = "one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\n";
        let after = "one\ntwo\nthree\nFOUR\nfive\nsix\nseven\neight\n";

        assert_eq!(
            rendered(before, after),
            [
                "@@ -1,7 +1,7 @@",
                "  one",
                "  two",
                "  three",
                "- four",
                "+ FOUR",
                "  five",
                "  six",
                "  seven",
            ]
        );
    }

    #[test]
    fn distant_changes_split_into_hunks() {
        let before = "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\n";
        let after = "A\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nL\n";

        assert_eq!(
            rendered(before, after),
            [
                "@@ -1,4 +1,4 @@",
                "- a",
                "+ A",
                "  b",
                "  c",
                "  d",
                "@@ -9,4 +9,4 @@",
                "  i",
                "  j",
                "  k",
                "- l",
                "+ L",
            ]
        );
    }

    #[test]
    fn a_missing_file_is_all_additions() {
        assert_eq!(
            rendered("", "font=monospace\n"),
            ["@@ -0,0 +1,1 @@", "+ font=monospace"]
        );
    }

    #[test]
    fn matching_content_has_no_hunk() {
        assert!(rendered("same\n", "same\n").is_empty());
    }

    #[test]
    fn a_lost_newline_is_named() {
        assert_eq!(
            rendered("tail\n", "tail"),
            [
                "@@ -1,1 +1,1 @@",
                "- tail",
                "+ tail",
                "\\ no newline at end of file",
            ]
        );
    }

    #[test]
    fn a_binary_file_is_not_read() {
        assert_eq!(readable(b"text\n"), Some("text\n"));
        assert_eq!(readable(b"\x00\x01"), None);
    }
}
