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
    fn literals_stay_out_of_the_generated_source() {
        let (source, literals) = compile(EXAMPLE).unwrap().into_parts();

        assert_eq!(literals, ["export EDITOR=", "\n", "path+=(", ")\n"]);
        assert!(!source.contains("export"));
        assert!(!source.contains("path"));
    }

    #[test]
    fn the_generated_source_has_the_documented_shape() {
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
    fn every_generated_line_matches_its_source_line() {
        let (source, _) = compile("1\n2\n3\n4\n5\n6\n<%= boom %>")
            .unwrap()
            .into_parts();

        assert_eq!(source.lines().nth(6).unwrap(), "__ld_write( boom );");
    }

    #[test]
    fn a_multi_line_tag_keeps_the_lines_that_follow_aligned() {
        let (source, literals) = compile("<% local x =\n1 %>done").unwrap().into_parts();

        assert_eq!(source, " local x =\n1 __ld_emit(1);");
        assert_eq!(literals, ["done"]);
    }

    #[test]
    fn a_comment_leaves_only_its_padding_behind() {
        let (source, _) = compile("<%# a note\nspanning lines %><%= x %>")
            .unwrap()
            .into_parts();

        assert_eq!(source, "\n__ld_write( x );");
    }
}
