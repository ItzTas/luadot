use std::fmt::Display;

use super::format::column;
use super::look::Look;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Stream {
    #[default]
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    head: String,
    tail: String,
    look: Look,
    mark: Option<String>,
    indent: usize,
    column: Option<usize>,
    blank: bool,
    stream: Stream,
    newline: bool,
}

impl Message {
    pub fn new(head: impl Display) -> Self {
        Self {
            head: head.to_string(),
            tail: String::new(),
            look: Look::default(),
            mark: None,
            indent: 0,
            column: None,
            blank: false,
            stream: Stream::Stdout,
            newline: true,
        }
    }

    pub fn with_tail(mut self, tail: impl Display) -> Self {
        self.tail = tail.to_string();
        self
    }

    pub fn with_look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }

    pub fn with_mark(mut self, mark: Option<String>) -> Self {
        self.mark = mark.or(self.mark);
        self
    }

    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    pub fn with_column(mut self, column: Option<usize>) -> Self {
        self.column = column.or(self.column);
        self
    }

    pub fn with_blank(mut self, blank: bool) -> Self {
        self.blank = blank;
        self
    }

    pub fn with_stream(mut self, stream: Stream) -> Self {
        self.stream = stream;
        self
    }

    pub fn with_newline(mut self, newline: bool) -> Self {
        self.newline = newline;
        self
    }

    pub fn look(&self) -> Look {
        self.look
    }

    pub fn stream(&self) -> Stream {
        self.stream
    }

    pub fn newline(&self) -> bool {
        self.newline
    }

    pub fn blank(&self) -> bool {
        self.blank
    }

    pub fn tail(&self) -> &str {
        &self.tail
    }

    pub fn indent(&self) -> String {
        " ".repeat(self.indent)
    }

    pub fn head(&self) -> String {
        let head = match &self.mark {
            Some(mark) => format!("{mark} {}", self.head),
            None => self.head.clone(),
        };

        match self.column {
            Some(width) => column(head, width),
            None => head,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mark_is_padded_by_its_head() {
        let message = Message::new("create")
            .with_mark(Some("»".to_string()))
            .with_column(Some(11));

        assert_eq!(message.head(), "» create   ");
    }
}
