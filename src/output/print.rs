use std::fmt::Display;
use std::io::Write;

use anstream::{eprintln, print, println, stdout};

use super::constants::{FIELD_WIDTH, LABEL_WIDTH};
use super::format::{column, notice};
use super::tone::Tone;

pub fn note(message: impl Display) {
    println!("{}", notice(message));
}

pub fn warn(message: impl Display) {
    let style = Tone::Warning.style();
    eprintln!("{style}{}{style:#}", notice(message));
}

pub fn error(message: impl Display) {
    let style = Tone::Bad.style();
    eprintln!("{style}{}{style:#}", notice(message));
}

pub fn line(text: impl Display) {
    println!("{text}");
}

pub fn section(title: impl Display) {
    let style = Tone::Strong.style();
    println!();
    println!("{style}{title}{style:#}");
}

pub fn entry(tone: Tone, label: impl Display, text: impl Display) {
    let style = tone.style();
    println!("{style}{}{style:#}{text}", column(label, LABEL_WIDTH));
}

pub fn field(name: impl Display, value: impl Display) {
    let style = Tone::Strong.style();
    println!("{style}{}{style:#}{value}", column(name, FIELD_WIDTH));
}

pub fn prompt(question: impl Display) -> std::io::Result<()> {
    print!("{question} ");
    stdout().flush()
}
