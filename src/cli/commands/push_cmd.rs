use anyhow::Result;

use super::git_cmd;

pub fn push_cmd(args: &[String]) -> Result<()> {
    git_cmd(&push_args(args))
}

fn push_args(args: &[String]) -> Vec<String> {
    let mut forwarded = vec!["push".to_string()];
    forwarded.extend_from_slice(args);
    forwarded
}

#[cfg(test)]
mod tests {
    use super::push_args;

    #[test]
    fn bare_push_forwards_only_push() {
        assert_eq!(push_args(&[]), ["push"]);
    }

    #[test]
    fn forwards_extra_args_after_push() {
        let args = ["origin".to_string(), "main".to_string()];
        assert_eq!(push_args(&args), ["push", "origin", "main"]);
    }
}
