use std::io::IsTerminal;

use anyhow::{Context, Result, bail};

use super::print::{line, prompt, warn};

pub fn confirm(command: &str, question: &str, skip: &str) -> Result<bool> {
    if !std::io::stdin().is_terminal() {
        bail!("{command}: cannot ask for confirmation without a terminal; pass {skip}");
    }

    ask(command, question)
}

pub fn offer(command: &str, question: &str) -> Result<bool> {
    if !std::io::stdin().is_terminal() {
        return Ok(false);
    }

    ask(command, question)
}

pub fn choose(
    command: &str,
    name: &str,
    question: &str,
    choices: &[String],
    default: Option<&str>,
    skip: &str,
) -> Result<String> {
    if !std::io::stdin().is_terminal() {
        bail!("{command}: cannot ask for `{name}` without a terminal; pass {skip}");
    }

    line(question);
    for (index, choice) in choices.iter().enumerate() {
        line(format!("  {}) {choice}", index + 1));
    }

    loop {
        prompt(format!("{}:", label(name, choices, default)))
            .with_context(|| format!("{command}: failed to write the question"))?;

        let answer = read_line(command, name)?;
        if let Some(value) = pick(&answer, choices, default) {
            return Ok(value);
        }

        warn(format!("`{}` is not one of the answers", answer.trim()));
    }
}

fn read_line(command: &str, name: &str) -> Result<String> {
    let mut answer = String::new();
    let read = std::io::stdin()
        .read_line(&mut answer)
        .with_context(|| format!("{command}: failed to read the answer"))?;

    if read == 0 {
        bail!("{command}: no answer for `{name}`");
    }

    Ok(answer)
}

fn label(name: &str, choices: &[String], default: Option<&str>) -> String {
    let hint = match (choices.is_empty(), default) {
        (false, None) => format!(" [1-{}]", choices.len()),
        (false, Some(default)) => format!(" [1-{}, enter for {default}]", choices.len()),
        (true, Some(default)) => format!(" [{default}]"),
        (true, None) => String::new(),
    };

    format!("{name}{hint}")
}

fn pick(answer: &str, choices: &[String], default: Option<&str>) -> Option<String> {
    let answer = answer.trim();
    if answer.is_empty() {
        return default.map(str::to_string);
    }
    if choices.is_empty() {
        return Some(answer.to_string());
    }
    if let Ok(number) = answer.parse::<usize>() {
        return choices.get(number.checked_sub(1)?).cloned();
    }

    choices.iter().find(|choice| *choice == answer).cloned()
}

fn ask(command: &str, question: &str) -> Result<bool> {
    prompt(format!("{question} [y/N]"))
        .with_context(|| format!("{command}: failed to write the question"))?;

    let mut answer = String::new();
    std::io::stdin()
        .read_line(&mut answer)
        .with_context(|| format!("{command}: failed to read the answer"))?;

    Ok(is_yes(&answer))
}

fn is_yes(answer: &str) -> bool {
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

#[cfg(test)]
mod tests {
    use super::{label, pick};

    fn choices() -> Vec<String> {
        vec!["desktop".to_string(), "laptop".to_string()]
    }

    #[test]
    fn a_choice_is_picked_by_number_or_name() {
        assert_eq!(pick("2", &choices(), None).unwrap(), "laptop");
        assert_eq!(pick(" laptop \n", &choices(), None).unwrap(), "laptop");
    }

    #[test]
    fn an_empty_answer_takes_the_default() {
        assert_eq!(pick("\n", &choices(), Some("laptop")).unwrap(), "laptop");
        assert_eq!(
            pick("  ", &[], Some("me@example.com")).unwrap(),
            "me@example.com"
        );
        assert_eq!(pick("\n", &choices(), None), None);
        assert_eq!(pick("\n", &[], None), None);
    }

    #[test]
    fn the_label_shows_the_answer_shape() {
        assert_eq!(label("form-factor", &choices(), None), "form-factor [1-2]");
        assert_eq!(
            label("form-factor", &choices(), Some("laptop")),
            "form-factor [1-2, enter for laptop]"
        );
        assert_eq!(
            label("email", &[], Some("me@example.com")),
            "email [me@example.com]"
        );
        assert_eq!(label("email", &[], None), "email");
    }
}
