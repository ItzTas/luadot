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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_customized_piece_names_the_command_and_the_call_it_came_from() {
        assert_eq!(
            customized("diff", "ld.on.diff", "summary"),
            "diff: `ld.on.diff`: `summary`"
        );
    }
}
