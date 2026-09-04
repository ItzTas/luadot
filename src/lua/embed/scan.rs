use anyhow::{Result, bail};

use super::constants::{CLOSE, OPEN};

#[derive(Debug, PartialEq, Eq)]
pub enum Segment {
    Literal { text: String, line: usize },
    Statement { lua: String, line: usize },
    Expression { lua: String, line: usize },
}

pub fn scan(source: &str) -> Result<Vec<Segment>> {
    Scanner {
        source,
        bytes: source.as_bytes(),
        pos: 0,
        line: 1,
    }
    .run()
}

enum Kind {
    Statement,
    Slurped,
    Expression,
    Comment,
}

enum Close {
    Plain,
    TrimNewline,
    Slurp,
}

struct Scanner<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    line: usize,
}

impl Scanner<'_> {
    fn run(mut self) -> Result<Vec<Segment>> {
        let mut segments = Vec::new();
        let mut literal = String::new();
        let mut literal_line = self.line;

        while self.pos < self.bytes.len() {
            if !self.starts_with(OPEN) {
                self.chunk_into(&mut literal);
                continue;
            }
            if self.starts_with("<%%") {
                self.advance(3);
                literal.push_str(OPEN);
                continue;
            }

            self.advance(OPEN.len());
            let opened = self.line;
            self.reject_unclaimed(opened)?;
            let kind = self.kind();
            if matches!(kind, Kind::Slurped) {
                slurp_before(&mut literal);
            }
            flush(&mut segments, &mut literal, literal_line);

            match kind {
                Kind::Comment => self.comment(opened)?,
                Kind::Expression => {
                    let (lua, close) = self.code(opened)?;
                    segments.push(Segment::Expression { lua, line: opened });
                    self.trim_after(close);
                }
                Kind::Statement | Kind::Slurped => {
                    let (lua, close) = self.code(opened)?;
                    segments.push(Segment::Statement { lua, line: opened });
                    self.trim_after(close);
                }
            }
            literal_line = self.line;
        }

        flush(&mut segments, &mut literal, literal_line);
        Ok(segments)
    }

    fn reject_unclaimed(&self, opened: usize) -> Result<()> {
        for spelling in ["|==", "|=", "|", "~", "=="] {
            if self.starts_with(spelling) {
                bail!("unsupported tag `{OPEN}{spelling}` opened on line {opened}");
            }
        }
        Ok(())
    }

    fn kind(&mut self) -> Kind {
        let kind = match self.bytes.get(self.pos) {
            Some(b'=' | b'-') => Kind::Expression,
            Some(b'_') => Kind::Slurped,
            Some(b'#') => Kind::Comment,
            _ => return Kind::Statement,
        };
        self.advance(1);
        kind
    }

    fn code(&mut self, opened: usize) -> Result<(String, Close)> {
        let mut lua = String::new();
        while self.pos < self.bytes.len() {
            if self.starts_with("%%>") {
                self.advance(3);
                lua.push_str(CLOSE);
                continue;
            }
            if self.starts_with("=%>") {
                bail!("unsupported closing tag `=%>` on line {}", self.line);
            }
            if self.starts_with("_%>") {
                self.advance(3);
                return Ok((lua, Close::Slurp));
            }
            if self.starts_with("-%>") {
                self.advance(3);
                return Ok((lua, Close::TrimNewline));
            }
            if self.starts_with(CLOSE) {
                self.advance(CLOSE.len());
                return Ok((lua, Close::Plain));
            }

            match self.bytes[self.pos] {
                quote @ (b'"' | b'\'') => self.string(&mut lua, quote),
                b'-' if self.bytes.get(self.pos + 1) == Some(&b'-') => self.comment_code(&mut lua),
                b'[' => match self.bracket_level() {
                    Some(level) => self.bracket(&mut lua, level),
                    None => self.push_code(&mut lua, 1),
                },
                _ => {
                    let len = self.char_len();
                    self.push_code(&mut lua, len);
                }
            }
        }
        bail!("unterminated tag opened on line {opened}")
    }

    fn comment(&mut self, opened: usize) -> Result<()> {
        while self.pos < self.bytes.len() {
            if self.starts_with("%%>") {
                self.advance(3);
                continue;
            }
            if self.starts_with(CLOSE) {
                self.advance(CLOSE.len());
                return Ok(());
            }
            self.advance(1);
        }
        bail!("unterminated tag opened on line {opened}")
    }

    fn string(&mut self, lua: &mut String, quote: u8) {
        let start = self.pos;
        let mut end = self.pos + 1;
        while end < self.bytes.len() {
            match self.bytes[end] {
                b'\\' => end = (end + 2).min(self.bytes.len()),
                b'\n' => break,
                byte if byte == quote => {
                    end += 1;
                    break;
                }
                _ => end += 1,
            }
        }
        lua.push_str(&self.source[start..end]);
        self.line += newlines(&self.bytes[start..end]);
        self.pos = end;
    }

    fn comment_code(&mut self, lua: &mut String) {
        self.push_code(lua, 2);
        if let Some(level) = self.bracket_level() {
            self.bracket(lua, level);
            return;
        }
        let start = self.pos;
        let mut end = self.pos;
        while end < self.bytes.len() && self.bytes[end] != b'\n' {
            end += 1;
        }
        lua.push_str(&self.source[start..end]);
        self.pos = end;
    }

    fn bracket_level(&self) -> Option<usize> {
        if self.bytes.get(self.pos) != Some(&b'[') {
            return None;
        }
        let mut level = 0;
        while self.bytes.get(self.pos + 1 + level) == Some(&b'=') {
            level += 1;
        }
        (self.bytes.get(self.pos + 1 + level) == Some(&b'[')).then_some(level)
    }

    fn bracket(&mut self, lua: &mut String, level: usize) {
        self.push_code(lua, level + 2);
        let closing = format!("]{}]", "=".repeat(level));
        while self.pos < self.bytes.len() {
            if self.starts_with(&closing) {
                self.push_code(lua, closing.len());
                return;
            }
            let len = self.char_len();
            self.push_code(lua, len);
        }
    }

    fn trim_after(&mut self, close: Close) {
        match close {
            Close::Plain => {}
            Close::Slurp => self.slurp_after(),
            Close::TrimNewline => self.trim_newline(),
        }
    }

    fn slurp_after(&mut self) {
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.advance(1);
        }
    }

    fn trim_newline(&mut self) {
        if self.bytes.get(self.pos) == Some(&b'\r') && self.bytes.get(self.pos + 1) == Some(&b'\n')
        {
            self.advance(2);
            return;
        }
        if self.bytes.get(self.pos) == Some(&b'\n') {
            self.advance(1);
        }
    }

    fn chunk_into(&mut self, literal: &mut String) {
        let start = self.pos;
        let mut end = self.pos + 1;
        while end < self.bytes.len() && self.bytes[end] != b'<' {
            end += 1;
        }
        literal.push_str(&self.source[start..end]);
        self.line += newlines(&self.bytes[start..end]);
        self.pos = end;
    }

    fn push_code(&mut self, lua: &mut String, count: usize) {
        lua.push_str(&self.source[self.pos..self.pos + count]);
        self.advance(count);
    }

    fn advance(&mut self, count: usize) {
        self.line += newlines(&self.bytes[self.pos..self.pos + count]);
        self.pos += count;
    }

    fn starts_with(&self, token: &str) -> bool {
        self.bytes[self.pos..].starts_with(token.as_bytes())
    }

    fn char_len(&self) -> usize {
        self.source[self.pos..]
            .chars()
            .next()
            .map_or(1, char::len_utf8)
    }
}

fn flush(segments: &mut Vec<Segment>, literal: &mut String, line: usize) {
    if literal.is_empty() {
        return;
    }
    segments.push(Segment::Literal {
        text: std::mem::take(literal),
        line,
    });
}

fn slurp_before(literal: &mut String) {
    while literal.ends_with([' ', '\t', '\r', '\n']) {
        literal.pop();
    }
}

fn newlines(bytes: &[u8]) -> usize {
    bytes.iter().filter(|byte| **byte == b'\n').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literal(text: &str, line: usize) -> Segment {
        Segment::Literal {
            text: text.to_string(),
            line,
        }
    }

    fn statement(lua: &str, line: usize) -> Segment {
        Segment::Statement {
            lua: lua.to_string(),
            line,
        }
    }

    fn expression(lua: &str, line: usize) -> Segment {
        Segment::Expression {
            lua: lua.to_string(),
            line,
        }
    }

    #[test]
    fn a_comment_produces_nothing() {
        assert_eq!(
            scan("a\n<%# a note\nspanning %>\nb").unwrap(),
            vec![literal("a\n", 1), literal("\nb", 3)]
        );
    }

    #[test]
    fn the_slurping_pair_welds_lines() {
        let source = "export A=1\n  <%_ for _, dir in ipairs(paths) do -%>\npath+=(<%= dir %>)\n  <%_ end -%>\n";

        assert_eq!(
            scan(source).unwrap(),
            vec![
                literal("export A=1", 1),
                statement(" for _, dir in ipairs(paths) do ", 2),
                literal("path+=(", 3),
                expression(" dir ", 3),
                literal(")", 3),
                statement(" end ", 4),
            ]
        );
    }

    #[test]
    fn a_close_inside_a_string_is_text() {
        assert_eq!(
            scan(r#"<% local s = "100%>" %>"#).unwrap(),
            vec![statement(r#" local s = "100%>" "#, 1)]
        );
        assert_eq!(
            scan("<% local s = '100%>' %>").unwrap(),
            vec![statement(" local s = '100%>' ", 1)]
        );
        assert_eq!(
            scan(r#"<% local s = "a\"%>" %>"#).unwrap(),
            vec![statement(r#" local s = "a\"%>" "#, 1)]
        );
    }

    #[test]
    fn an_unclaimed_spelling_is_an_error() {
        for source in [
            "<%~ x %>",
            "<%| x %>",
            "<%|= x %>",
            "<%|== x %>",
            "<%== x %>",
        ] {
            let err = scan(source).unwrap_err().to_string();
            assert!(err.contains("unsupported tag"), "{source}: {err}");
            assert!(err.contains("line 1"), "{source}: {err}");
        }

        let err = scan("a\n<%= x =%>").unwrap_err().to_string();
        assert!(err.contains("`=%>`"));
        assert!(err.contains("line 2"));
    }
}
