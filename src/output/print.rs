use std::fmt::Display;
use std::io::Write;

use anstream::{eprint, eprintln, print, println, stderr, stdout};

use super::constants::{FIELD_WIDTH, ITEM_WIDTH, LABEL_WIDTH};
use super::format::notice;
use super::message::{Message, Stream};
use super::tone::Tone;

const ITEM_INDENT: usize = 8;

const HINT_INDENT: usize = 2;

pub fn say(message: &Message) {
    if message.blank() {
        write(message.stream(), true, String::new());
    }

    let style = message.look().style();
    write(
        message.stream(),
        message.newline(),
        format!(
            "{}{style}{}{style:#}{}",
            message.indent(),
            message.head(),
            message.tail()
        ),
    );
}

pub fn note(message: impl Display) {
    say(&Message::new(notice(message)));
}

pub fn warn(message: impl Display) {
    say(&Message::new(notice(message))
        .with_look(Tone::Warning.into())
        .with_stream(Stream::Stderr));
}

pub fn error(message: impl Display) {
    say(&Message::new(notice(message))
        .with_look(Tone::Bad.into())
        .with_stream(Stream::Stderr));
}

pub fn line(text: impl Display) {
    say(&Message::new(text));
}

pub fn title(text: impl Display) {
    say(&Message::new(text).with_look(Tone::Strong.into()));
}

pub fn section(title: impl Display) {
    say(&Message::new(title)
        .with_look(Tone::Strong.into())
        .with_blank(true));
}

pub fn hint(text: impl Display) {
    say(&muted(text));
}

pub fn detail(text: impl Display) {
    say(&muted(text));
}

fn muted(text: impl Display) -> Message {
    Message::new(text)
        .with_look(Tone::Muted.into())
        .with_indent(HINT_INDENT)
}

pub fn item(tone: Tone, label: impl Display, text: impl Display) {
    say(&Message::new(label)
        .with_tail(text)
        .with_look(tone.into())
        .with_indent(ITEM_INDENT)
        .with_column(Some(ITEM_WIDTH)));
}

pub fn entry(tone: Tone, label: impl Display, text: impl Display) {
    say(&Message::new(label)
        .with_tail(text)
        .with_look(tone.into())
        .with_column(Some(LABEL_WIDTH)));
}

pub fn field(name: impl Display, value: impl Display) {
    say(&Message::new(name)
        .with_tail(value)
        .with_look(Tone::Strong.into())
        .with_column(Some(FIELD_WIDTH)));
}

pub fn prompt(question: impl Display) -> std::io::Result<()> {
    print!("{question} ");
    stdout().flush()
}

fn write(stream: Stream, newline: bool, body: String) {
    if newline {
        match stream {
            Stream::Stdout => println!("{body}"),
            Stream::Stderr => eprintln!("{body}"),
        }
        return;
    }

    match stream {
        Stream::Stdout => {
            print!("{body}");
            let _ = stdout().flush();
        }
        Stream::Stderr => {
            eprint!("{body}");
            let _ = stderr().flush();
        }
    }
}
