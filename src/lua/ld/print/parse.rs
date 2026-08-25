use anstyle::{Ansi256Color, Color, RgbColor};
use mlua::{Function, Lua, Table, Value};

use super::super::constants::API;
use super::super::parse::{external, lookup};
use super::constants::{
    BG, BOLD, COLORS, DATE, DIM, FG, HEX, HEX_DIGITS, INDENT, ITALIC, MARK, NEWLINE, OPTIONS, OS,
    SHADES, STREAM, STREAMS, TIME, TIME_FORMAT, TONE, TONES, UNDERLINE, WIDTH,
};
use crate::output::{Look, Message, Stream, Tone};

type Effect = fn(Look, Option<bool>) -> Look;

pub fn text(call: &str, value: &Value) -> mlua::Result<String> {
    match value {
        Value::Nil => Ok(String::new()),
        Value::String(text) => Ok(text.to_str()?.to_string()),
        Value::Integer(number) => Ok(number.to_string()),
        Value::Number(number) => Ok(number.to_string()),
        other => Err(external(format!(
            "`{API}.{call}` takes a string, got {}",
            other.type_name()
        ))),
    }
}

pub fn message(
    lua: &Lua,
    call: &str,
    base: Message,
    options: Option<Table>,
) -> mlua::Result<Message> {
    let Some(options) = options else {
        return Ok(base);
    };
    known(call, &options)?;

    let look = look(call, &options, base.look())?;
    let mut message = base
        .with_look(look)
        .with_mark(mark(lua, call, &options)?)
        .with_column(count(call, &options, WIDTH)?);

    if let Some(indent) = count(call, &options, INDENT)? {
        message = message.with_indent(indent);
    }
    if let Some(stream) = stream(call, &options)? {
        message = message.with_stream(stream);
    }
    if let Some(newline) = flag(call, &options, NEWLINE)? {
        message = message.with_newline(newline);
    }

    Ok(message)
}

fn look(call: &str, options: &Table, base: Look) -> mlua::Result<Look> {
    let mut look = base
        .with_tone(tone(call, options)?)
        .with_fg(color(call, options, FG)?)
        .with_bg(color(call, options, BG)?);

    let effects: [(&str, Effect); 4] = [
        (BOLD, Look::with_bold),
        (DIM, Look::with_dim),
        (ITALIC, Look::with_italic),
        (UNDERLINE, Look::with_underline),
    ];
    for (key, apply) in effects {
        look = apply(look, flag(call, options, key)?);
    }

    Ok(look)
}

fn tone(call: &str, options: &Table) -> mlua::Result<Option<Tone>> {
    let Some(name) = name(call, options, TONE)? else {
        return Ok(None);
    };

    lookup(&TONES, &name, "tone").map(Some)
}

fn stream(call: &str, options: &Table) -> mlua::Result<Option<Stream>> {
    let Some(name) = name(call, options, STREAM)? else {
        return Ok(None);
    };

    lookup(&STREAMS, &name, "stream").map(Some)
}

fn color(call: &str, options: &Table, key: &str) -> mlua::Result<Option<Color>> {
    match options.get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::Integer(code) => shade(call, key, code).map(Some),
        Value::String(text) => written(call, key, text.to_str()?.as_ref()).map(Some),
        other => Err(expected(
            call,
            key,
            &format!("a color name, {SHADES} or a hex color like \"#ff8800\""),
            &other,
        )),
    }
}

fn shade(call: &str, key: &str, code: i64) -> mlua::Result<Color> {
    u8::try_from(code)
        .map(|code| Color::Ansi256(Ansi256Color(code)))
        .map_err(|_| {
            external(format!(
                "`{API}.{call}`: `{key}` takes {SHADES}, got {code}"
            ))
        })
}

fn written(call: &str, key: &str, text: &str) -> mlua::Result<Color> {
    let Some(digits) = text.strip_prefix(HEX) else {
        return lookup(&COLORS, text, "color").map(Color::Ansi);
    };

    hexed(digits).ok_or_else(|| {
        external(format!(
            "`{API}.{call}`: `{key}` takes a hex color like \"#ff8800\", got `{text}`"
        ))
    })
}

fn hexed(digits: &str) -> Option<Color> {
    let written =
        digits.len() == HEX_DIGITS && digits.bytes().all(|digit| digit.is_ascii_hexdigit());
    if !written {
        return None;
    }

    let channel = |at: usize| u8::from_str_radix(&digits[at..at + 2], 16).ok();

    Some(Color::Rgb(RgbColor(channel(0)?, channel(2)?, channel(4)?)))
}

fn mark(lua: &Lua, call: &str, options: &Table) -> mlua::Result<Option<String>> {
    let stamp = time(lua, call, options)?;
    let mark = match options.get::<Value>(MARK)? {
        Value::Nil => None,
        Value::String(text) => Some(text.to_str()?.to_string()),
        Value::Function(function) => Some(produced(call, MARK, &function)?),
        other => Err(expected(call, MARK, "a string or a function", &other))?,
    };

    Ok(match (stamp, mark) {
        (None, mark) => mark,
        (Some(stamp), None) => Some(stamp),
        (Some(stamp), Some(mark)) => Some(format!("{stamp} {mark}")),
    })
}

fn time(lua: &Lua, call: &str, options: &Table) -> mlua::Result<Option<String>> {
    let format = match options.get::<Value>(TIME)? {
        Value::Nil | Value::Boolean(false) => return Ok(None),
        Value::Boolean(true) => TIME_FORMAT.to_string(),
        Value::String(format) => format.to_str()?.to_string(),
        other => Err(expected(
            call,
            TIME,
            "true or a strftime format like \"%H:%M\"",
            &other,
        ))?,
    };

    let date: Function = lua.globals().get::<Table>(OS)?.get(DATE)?;

    date.call::<String>(format.clone()).map(Some).map_err(|_| {
        external(format!(
            "`{API}.{call}`: `{TIME}` takes a strftime format holding text, got `{format}`"
        ))
    })
}

fn produced(call: &str, key: &str, function: &Function) -> mlua::Result<String> {
    match function.call::<Value>(())? {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        other => Err(external(format!(
            "`{API}.{call}`: `{key}` returned {}; a string is expected",
            other.type_name()
        ))),
    }
}

fn name(call: &str, options: &Table, key: &str) -> mlua::Result<Option<String>> {
    match options.get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::String(name) => Ok(Some(name.to_str()?.to_string())),
        other => Err(expected(call, key, "a string", &other)),
    }
}

fn flag(call: &str, options: &Table, key: &str) -> mlua::Result<Option<bool>> {
    match options.get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::Boolean(flag) => Ok(Some(flag)),
        other => Err(expected(call, key, "true or false", &other)),
    }
}

fn count(call: &str, options: &Table, key: &str) -> mlua::Result<Option<usize>> {
    let whole = match options.get::<Value>(key)? {
        Value::Nil => return Ok(None),
        Value::Integer(count) => count,
        Value::Number(count) if count.fract() == 0.0 => count as i64,
        other => Err(expected(call, key, "a whole number", &other))?,
    };

    usize::try_from(whole).map(Some).map_err(|_| {
        external(format!(
            "`{API}.{call}`: `{key}` takes a whole number of zero or more, got {whole}"
        ))
    })
}

fn known(call: &str, options: &Table) -> mlua::Result<()> {
    for pair in options.clone().pairs::<String, Value>() {
        let (key, _) =
            pair.map_err(|_| external(format!("`{API}.{call}` takes a table of options")))?;

        if !OPTIONS.contains(&key.as_str()) {
            return Err(external(format!(
                "`{API}.{call}`: unknown option `{key}` (available: {})",
                OPTIONS.join(", ")
            )));
        }
    }

    Ok(())
}

fn expected(call: &str, key: &str, kind: &str, value: &Value) -> mlua::Error {
    external(format!(
        "`{API}.{call}`: `{key}` takes {kind}, got {}",
        value.type_name()
    ))
}

#[cfg(test)]
mod tests {
    use anstyle::AnsiColor;

    use super::super::constants::NAMESPACE;
    use super::*;
    use crate::lua::runtime::runtime;

    fn options(lua: &Lua, source: &str) -> Table {
        lua.load(source).eval().unwrap()
    }

    fn built(source: &str) -> mlua::Result<Message> {
        let lua = runtime().unwrap();
        let options = options(&lua, source);

        message(&lua, NAMESPACE, Message::new("text"), Some(options))
    }

    #[test]
    fn a_color_is_read_three_ways() {
        assert_eq!(
            built(r#"return { fg = "cyan" }"#)
                .unwrap()
                .look()
                .style()
                .get_fg_color(),
            Some(AnsiColor::Cyan.into())
        );
        assert_eq!(
            built("return { fg = 213 }")
                .unwrap()
                .look()
                .style()
                .get_fg_color(),
            Some(Color::Ansi256(Ansi256Color(213)))
        );
        assert_eq!(
            built(r##"return { fg = "#ff8800" }"##)
                .unwrap()
                .look()
                .style()
                .get_fg_color(),
            Some(Color::Rgb(RgbColor(255, 136, 0)))
        );
    }

    #[test]
    fn stream_indent_and_newline_are_read() {
        let message =
            built(r#"return { stream = "stderr", indent = 2, newline = false }"#).unwrap();

        assert_eq!(message.stream(), Stream::Stderr);
        assert_eq!(message.indent(), "  ");
        assert!(!message.newline());
    }

    #[test]
    fn rejects_an_unknown_option() {
        let err = built(r#"return { colour = "red" }"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("unknown option `colour`"));
        assert!(err.contains("available: bg, bold, dim, fg"));
    }

    #[test]
    fn an_empty_mark_is_reported() {
        let err = built("return { mark = function() end }")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`mark` returned nil; a string is expected"));
    }
}
