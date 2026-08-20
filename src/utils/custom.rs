use crate::output;

pub fn customized(command: &str, call: &str, key: &str) -> String {
    format!("{command}: `{call}`: `{key}`")
}

pub fn said(shown: Option<String>) {
    let Some(text) = shown else {
        return;
    };

    output::line(text);
}
