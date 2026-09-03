use mlua::{Function, Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::external;
use super::constants::{CONCAT, CONTENT, NAMESPACE, SEPARATOR, WHEN};

pub fn function(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, (sections, separator): (Value, Option<String>)| {
        concat(&sections, separator.as_deref().unwrap_or(SEPARATOR))
    })
}

fn concat(sections: &Value, separator: &str) -> mlua::Result<String> {
    let Some(list) = sections.as_table() else {
        return Err(external(format!(
            "`{API}.{NAMESPACE}.{CONCAT}` takes a list of sections, got {}",
            sections.type_name()
        )));
    };

    let mut parts = Vec::new();
    for (index, section) in list.clone().sequence_values::<Value>().enumerate() {
        if let Some(part) = fragment(&section?, index + 1)? {
            parts.push(part);
        }
    }

    Ok(parts.join(separator))
}

fn fragment(section: &Value, ordinal: usize) -> mlua::Result<Option<String>> {
    if let Some(text) = section.as_string() {
        return Ok(Some(text.to_str()?.to_string()));
    }

    let Some(section) = section.as_table() else {
        return Err(numbered(
            ordinal,
            format!("is a {}, not a string or a table", section.type_name()),
        ));
    };

    known(section, ordinal)?;
    if !included(section, ordinal)? {
        return Ok(None);
    }

    content(section, ordinal).map(Some)
}

fn content(section: &Table, ordinal: usize) -> mlua::Result<String> {
    match section.get::<Value>(CONTENT)? {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        Value::Nil => Err(numbered(ordinal, format!("needs a `{CONTENT}`"))),
        other => Err(numbered(
            ordinal,
            format!(
                "needs a `{CONTENT}` holding a string, got {}",
                other.type_name()
            ),
        )),
    }
}

fn included(section: &Table, ordinal: usize) -> mlua::Result<bool> {
    match section.get::<Value>(WHEN)? {
        Value::Nil => Ok(true),
        Value::Boolean(when) => Ok(when),
        other => Err(numbered(
            ordinal,
            format!(
                "needs a `{WHEN}` holding true or false, got {}",
                other.type_name()
            ),
        )),
    }
}

fn known(section: &Table, ordinal: usize) -> mlua::Result<()> {
    for pair in section.clone().pairs::<String, Value>() {
        let (key, _) = pair.map_err(|_| numbered(ordinal, "takes a table of keys"))?;

        if key != CONTENT && key != WHEN {
            return Err(numbered(
                ordinal,
                format!("has an unknown key `{key}` (available: {CONTENT}, {WHEN})"),
            ));
        }
    }

    Ok(())
}

fn numbered(ordinal: usize, message: impl AsRef<str>) -> mlua::Error {
    external(format!(
        "`{API}.{NAMESPACE}.{CONCAT}` section {ordinal} {}",
        message.as_ref()
    ))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::super::fixture::{error, template};
    use crate::lua::{Content, from_template};

    fn write(dir: &Path, name: &str, contents: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    fn built(dir: &Path, source: &str) -> String {
        let outputs = from_template(dir, source).unwrap();

        match outputs[0].content() {
            Content::Text(text) => text.clone(),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn joins_what_the_other_calls_return() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        write(&dir, "conf.d/10-env.zsh", "export EDITOR=nvim");
        write(&dir, "conf.d/20-path.zsh", "path+=(~/bin)");
        write(&dir, "prompt.tmpl.lua", r#"return "PS1=" .. mark"#);
        write(&dir, "greet.tmpl.zsh", "echo <%= who %>");

        let built = built(
            &dir,
            r##"
            local parts = {}
            for _, name in ipairs(ld.alt.glob("conf.d/*.zsh")) do
              parts[#parts + 1] = ld.alt.read(name)
            end

            return ld.alt.concat({
              "# built by luadot",
              ld.alt.concat(parts),
              ld.alt.render("prompt.tmpl.lua", { mark = "%" }),
              { content = ld.alt.expand("greet.tmpl.zsh", { who = "world" }) },
              ld.alt.json({ editor = "nvim" }),
            })
            "##,
        );

        assert_eq!(
            built,
            "# built by luadot\nexport EDITOR=nvim\npath+=(~/bin)\nPS1=%\necho world\n{\n  \"editor\": \"nvim\"\n}"
        );
    }

    #[test]
    fn a_false_when_leaves_its_section_out() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        write(&dir, "laptop.zsh", "laptop");
        write(&dir, "server.zsh", "server");

        let built = built(
            &dir,
            r#"
            return ld.alt.concat({
              { content = ld.alt.read("laptop.zsh"), when = ld.alt.exists("laptop.zsh") },
              { content = ld.alt.read("server.zsh"), when = false },
            })
            "#,
        );

        assert_eq!(built, "laptop");
    }

    #[test]
    fn the_separator_joins_what_it_is_given() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        write(&dir, "aliases.zsh", "alias ll='ls -l'\n");

        let built = built(
            &dir,
            r##"return ld.alt.concat({ ld.alt.read("aliases.zsh"), "# end\n" }, "")"##,
        );

        assert_eq!(built, "alias ll='ls -l'\n# end\n");
    }

    #[test]
    fn a_broken_section_says_which_one() {
        let root = tempfile::tempdir().unwrap();
        let dir = template(root.path());
        write(&dir, "aliases.zsh", "alias ll='ls -l'");

        let cases = [
            (r#""ok", { when = true }"#, "section 2 needs a `content`"),
            (
                r#"{ content = ld.alt.file("aliases.zsh") }"#,
                "section 1 needs a `content` holding a string, got userdata",
            ),
            (
                r#"{ content = "x", read = "y" }"#,
                "section 1 has an unknown key `read` (available: content, when)",
            ),
            (
                r#"{ content = "x", when = "yes" }"#,
                "section 1 needs a `when` holding true or false, got string",
            ),
            (
                r#""ok", true"#,
                "section 2 is a boolean, not a string or a table",
            ),
        ];

        for (sections, expected) in cases {
            let err = error(&dir, &format!("return ld.alt.concat({{ {sections} }})"));

            assert!(err.contains(expected), "{sections} answered with {err}");
        }
    }
}
