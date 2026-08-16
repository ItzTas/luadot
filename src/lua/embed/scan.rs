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
    Trimmed,
    Expression,
    Comment,
}

enum Close {
    Plain,
    TrimNewline,
    TrimSpaces,
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
            let kind = self.kind();
            if matches!(kind, Kind::Trimmed) {
                trim_indent(&mut literal);
            }
            flush(&mut segments, &mut literal, literal_line);

            match kind {
                Kind::Comment => self.comment(opened)?,
                Kind::Expression => {
                    let (lua, close) = self.code(opened)?;
                    segments.push(Segment::Expression { lua, line: opened });
                    self.trim_after(close);
                }
                Kind::Statement | Kind::Trimmed => {
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

    fn kind(&mut self) -> Kind {
        let kind = match self.bytes.get(self.pos) {
            Some(b'=' | b'-') => Kind::Expression,
            Some(b'_') => Kind::Trimmed,
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
            if self.starts_with("_%>") {
                self.advance(3);
                return Ok((lua, Close::TrimSpaces));
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
        if matches!(close, Close::Plain) {
            return;
        }
        if matches!(close, Close::TrimSpaces) {
            while matches!(self.bytes.get(self.pos), Some(b' ' | b'\t')) {
                self.pos += 1;
            }
        }
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

fn trim_indent(literal: &mut String) {
    while literal.ends_with(' ') || literal.ends_with('\t') {
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
    fn text_and_tags_split_into_segments() {
        assert_eq!(
            scan("a<% x = 1 %>b<%= x %>c").unwrap(),
            vec![
                literal("a", 1),
                statement(" x = 1 ", 1),
                literal("b", 1),
                expression(" x ", 1),
                literal("c", 1),
            ]
        );
    }

    #[test]
    fn the_dash_open_is_an_alias_of_the_output_tag() {
        assert_eq!(scan("<%- x %>").unwrap(), scan("<%= x %>").unwrap());
    }

    #[test]
    fn a_comment_produces_nothing_and_keeps_the_lines() {
        assert_eq!(
            scan("a\n<%# a note\nspanning %>\nb").unwrap(),
            vec![literal("a\n", 1), literal("\nb", 3)]
        );
    }

    #[test]
    fn a_comment_closes_on_the_plain_delimiter_alone() {
        assert_eq!(scan("<%# note -%>\nb").unwrap(), vec![literal("\nb", 1)]);
    }

    #[test]
    fn the_underscore_open_trims_the_indentation_before_the_tag() {
        assert_eq!(
            scan("a\n  \t<%_ x = 1 %>").unwrap(),
            vec![literal("a\n", 1), statement(" x = 1 ", 2)]
        );
    }

    #[test]
    fn the_dash_close_trims_the_newline_after_the_tag() {
        assert_eq!(
            scan("a\n<% x = 1 -%>\nb").unwrap(),
            vec![literal("a\n", 1), statement(" x = 1 ", 2), literal("b", 3)]
        );
    }

    #[test]
    fn the_underscore_close_trims_the_spaces_and_the_newline_after_the_tag() {
        assert_eq!(
            scan("<% x = 1 _%>  \t\nb").unwrap(),
            vec![statement(" x = 1 ", 1), literal("b", 2)]
        );
    }

    #[test]
    fn an_indented_block_keeps_its_lines_intact() {
        let source = "export A=1\n  <%_ for _, dir in ipairs(paths) do -%>\npath+=(<%= dir %>)\n  <%_ end -%>\n";

        assert_eq!(
            scan(source).unwrap(),
            vec![
                literal("export A=1\n", 1),
                statement(" for _, dir in ipairs(paths) do ", 2),
                literal("path+=(", 3),
                expression(" dir ", 3),
                literal(")\n", 3),
                statement(" end ", 4),
            ]
        );
    }

    #[test]
    fn the_open_escape_is_a_literal() {
        assert_eq!(scan("a<%%= x %>b").unwrap(), vec![literal("a<%= x %>b", 1)]);
    }

    #[test]
    fn the_close_escape_is_code() {
        assert_eq!(
            scan("<% x = a %%> b %>").unwrap(),
            vec![statement(" x = a %> b ", 1)]
        );
    }

    #[test]
    fn a_close_inside_a_lua_string_does_not_close() {
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
    fn a_close_inside_a_lua_comment_does_not_close() {
        assert_eq!(
            scan("<% x = 1 -- %> still comment\nx = 2 %>").unwrap(),
            vec![statement(" x = 1 -- %> still comment\nx = 2 ", 1)]
        );
        assert_eq!(
            scan("<% --[[ %> ]] x = 1 %>").unwrap(),
            vec![statement(" --[[ %> ]] x = 1 ", 1)]
        );
    }

    #[test]
    fn a_close_inside_a_long_bracket_does_not_close() {
        assert_eq!(
            scan("<% s = [==[%> ]] ]==] %>").unwrap(),
            vec![statement(" s = [==[%> ]] ]==] ", 1)]
        );
    }

    #[test]
    fn an_unterminated_tag_reports_the_line_it_was_opened_on() {
        let err = scan("line one\n<% x = 1").unwrap_err().to_string();
        assert!(err.contains("unterminated tag"));
        assert!(err.contains("line 2"));

        let err = scan("<%# never closed").unwrap_err().to_string();
        assert!(err.contains("line 1"));
    }
}
