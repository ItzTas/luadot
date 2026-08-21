use mlua::{Function, Lua, Value};
use regex::{Captures, Regex};

use super::super::parse::external;
use super::captures::{owned, values};
use super::constants::{GSUB, PATTERN, TEXT};
use super::parse::{compile, limit, prefix, text};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(
        |lua, (subject, pattern, replacement, times): (Value, Value, Value, Value)| {
            let subject = text(&subject, GSUB, TEXT)?;
            let pattern = text(&pattern, GSUB, PATTERN)?;
            let times = limit(&times, GSUB)?;
            let regex = compile(&pattern, GSUB)?;

            match &replacement {
                Value::String(template) => {
                    let template = template.to_str()?.to_string();
                    replace(&subject, &regex, times, |captures| {
                        let mut piece = String::new();
                        captures.expand(&template, &mut piece);
                        Ok(piece)
                    })
                }
                Value::Function(build) => replace(&subject, &regex, times, |captures| {
                    piece(lua, build, captures)
                }),
                _ => Err(external(format!(
                    "{} takes the replacement as a string or a function",
                    prefix(GSUB)
                ))),
            }
        },
    )
}

fn piece(lua: &Lua, build: &Function, captures: &Captures) -> mlua::Result<String> {
    let whole = captures[0].to_string();
    let produced: Value = build.call(values(lua, &owned(captures))?)?;

    match produced {
        Value::String(piece) => Ok(piece.to_str()?.to_string()),
        Value::Nil | Value::Boolean(false) => Ok(whole),
        _ => Err(external(format!(
            "{} takes a replacement function returning a string",
            prefix(GSUB)
        ))),
    }
}

fn replace(
    subject: &str,
    regex: &Regex,
    times: usize,
    mut build: impl FnMut(&Captures) -> mlua::Result<String>,
) -> mlua::Result<(String, usize)> {
    let mut replaced = String::with_capacity(subject.len());
    let mut last = 0;
    let mut done = 0;

    for captures in regex.captures_iter(subject) {
        if times != 0 && done == times {
            break;
        }

        let whole = captures.get(0).expect("a match carries its whole text");
        replaced.push_str(&subject[last..whole.start()]);
        replaced.push_str(&build(&captures)?);
        last = whole.end();
        done += 1;
    }
    replaced.push_str(&subject[last..]);

    Ok((replaced, done))
}

#[cfg(test)]
mod tests {
    use super::super::fixture::eval;

    #[test]
    fn rewrites_every_match_and_says_how_many() {
        assert_eq!(
            eval(
                r#"
                local text, count = regex.gsub("a=1, b=2", "\\d", "0")
                return text .. "|" .. count
                "#
            )
            .unwrap(),
            "a=0, b=0|2"
        );
    }

    #[test]
    fn a_limit_stops_after_that_many_matches() {
        assert_eq!(
            eval(r#"return regex.gsub("a=1, b=2, c=3", "\\d", "0", 2)"#).unwrap(),
            "a=0, b=0, c=3"
        );
    }

    #[test]
    fn a_function_builds_each_replacement_out_of_the_match() {
        assert_eq!(
            eval(
                r#"
                return regex.gsub("a=1, b=2", "(\\w)=(\\d)", function(_, key, value)
                  return key .. ":" .. (tonumber(value) + 1)
                end)
                "#
            )
            .unwrap(),
            "a:2, b:3"
        );
    }

    #[test]
    fn a_function_yielding_nothing_keeps_the_match() {
        assert_eq!(
            eval(
                r#"
                return regex.gsub("a=1, b=2", "\\d", function(whole)
                  if whole == "1" then return nil end
                  return "9"
                end)
                "#
            )
            .unwrap(),
            "a=1, b=9"
        );
    }

    #[test]
    fn reports_a_replacement_that_is_neither_a_string_nor_a_function() {
        let err = eval(r#"return regex.gsub("a=1", "\\d", 1)"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("`ld.regex.gsub` takes the replacement as a string or a function"));
    }
}
