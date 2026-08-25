use anyhow::Result;

use super::constants::{EMIT, WRITE};
use super::scan::{Segment, scan};

#[derive(Debug, PartialEq, Eq)]
pub struct Chunk {
    source: String,
    literals: Vec<String>,
}

impl Chunk {
    pub fn into_parts(self) -> (String, Vec<String>) {
        (self.source, self.literals)
    }
}

pub fn compile(source: &str) -> Result<Chunk> {
    Ok(assemble(scan(source)?))
}

fn assemble(segments: Vec<Segment>) -> Chunk {
    let mut source = String::new();
    let mut literals = Vec::new();
    let mut line = 1;

    for segment in segments {
        match segment {
            Segment::Literal { text, line: at } => {
                pad(&mut source, &mut line, at);
                literals.push(text);
                append(
                    &mut source,
                    &mut line,
                    &format!("{EMIT}({});", literals.len()),
                );
            }
            Segment::Expression { lua, line: at } => {
                pad(&mut source, &mut line, at);
                append(&mut source, &mut line, &format!("{WRITE}({lua});"));
            }
            Segment::Statement { lua, line: at } => {
                pad(&mut source, &mut line, at);
                append(&mut source, &mut line, &lua);
            }
        }
    }

    Chunk { source, literals }
}

fn pad(source: &mut String, line: &mut usize, target: usize) {
    while *line < target {
        source.push('\n');
        *line += 1;
    }
}

fn append(source: &mut String, line: &mut usize, code: &str) {
    *line += code.matches('\n').count();
    source.push_str(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "export EDITOR=<%= ld.class.get(\"editor\") or \"nvim\" %>\n<% for _, dir in ipairs({ \"~/bin\", \"~/.local/bin\" }) do -%>\npath+=(<%= dir %>)\n<% end -%>\n";

    #[test]
    fn generated_source_matches_the_docs() {
        let (source, _) = compile(EXAMPLE).unwrap().into_parts();
        let lines: Vec<&str> = source.lines().collect();

        assert_eq!(
            lines,
            vec![
                "__ld_emit(1);__ld_write( ld.class.get(\"editor\") or \"nvim\" );__ld_emit(2);",
                " for _, dir in ipairs({ \"~/bin\", \"~/.local/bin\" }) do ",
                "__ld_emit(3);__ld_write( dir );__ld_emit(4);",
                " end ",
            ]
        );
    }

    #[test]
    fn a_multi_line_tag_keeps_alignment() {
        let (source, literals) = compile("<% local x =\n1 %>done").unwrap().into_parts();

        assert_eq!(source, " local x =\n1 __ld_emit(1);");
        assert_eq!(literals, ["done"]);
    }
}
