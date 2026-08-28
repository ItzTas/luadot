use tealr::{
    EnumGenerator, ExportedFunction, FunctionParam, RecordGenerator, Type, TypeGenerator,
    TypeWalker,
};

const HEADER: &str = "---@meta";

const COMMENT: &str = "---";

const ALIAS: &str = "---@alias";

const VARIANT: &str = "---|";

const CLASS: &str = "---@class";

const FIELD: &str = "---@field";

const OVERLOAD: &str = "---@overload";

const PARAM: &str = "---@param";

const RETURN: &str = "---@return";

const FUNCTION: &str = "function";

const END: &str = "end";

const FUN: &str = "fun";

const EMPTY_TABLE: &str = "= {}";

const ELLIPSIS: &str = "...";

const OPTIONAL: &str = "?";

const UNNAMED: &str = "_";

const ARRAY: &str = "[]";

const MAP: &str = "table";

use crate::lua::ld::{CALL_METHOD, NIL};

struct Union {
    text: String,
    optional: bool,
    count: usize,
}

pub fn render(walker: &TypeWalker) -> String {
    let mut lines = vec![HEADER.to_string()];

    for choices in walker.given_types.iter().filter_map(enumeration) {
        lines.push(String::new());
        alias(&mut lines, choices);
    }
    for record in walker.given_types.iter().filter_map(TypeGenerator::record) {
        if instanced(walker, &record.ty) {
            continue;
        }
        lines.push(String::new());
        class(&mut lines, record);
    }
    for instance in &walker.global_instances_off {
        let Some(record) = record_of(walker, &instance.ty) else {
            continue;
        };
        lines.push(String::new());
        class(&mut lines, record);
        lines.push(format!("{} {EMPTY_TABLE}", instance.name));

        for function in &record.functions {
            lines.push(String::new());
            definition(&mut lines, &instance.name, record, function);
        }
    }

    lines.join("\n") + "\n"
}

fn enumeration(generator: &TypeGenerator) -> Option<&EnumGenerator> {
    match generator {
        TypeGenerator::Enum(choices) => Some(choices),
        TypeGenerator::Record(_) => None,
    }
}

fn record_of<'a>(walker: &'a TypeWalker, ty: &Type) -> Option<&'a RecordGenerator> {
    walker
        .given_types
        .iter()
        .filter_map(TypeGenerator::record)
        .find(|record| &record.ty == ty)
}

fn instanced(walker: &TypeWalker, ty: &Type) -> bool {
    walker
        .global_instances_off
        .iter()
        .any(|instance| &instance.ty == ty)
}

fn comment(lines: &mut Vec<String>, text: &str) {
    for line in text.lines() {
        lines.push(format!("{COMMENT}{line}"));
    }
}

fn alias(lines: &mut Vec<String>, choices: &EnumGenerator) {
    comment(lines, &choices.type_doc);
    lines.push(format!("{ALIAS} {}", choices.name));
    for variant in &choices.variants {
        lines.push(format!("{VARIANT} \"{variant}\""));
    }
}

fn class(lines: &mut Vec<String>, record: &RecordGenerator) {
    comment(lines, &record.type_doc);
    lines.push(format!("{CLASS} {}", typename(&record.ty)));

    for field in &record.fields {
        let union = unwrapped(&field.ty);
        let mut line = format!(
            "{FIELD} {}{} {}",
            field.name,
            suffix(union.optional),
            union.text
        );
        if let Some(doc) = record.documentation.get(&field.name) {
            line.push(' ');
            line.push_str(&doc.replace('\n', " "));
        }
        lines.push(line);
    }
    for call in record
        .meta_function
        .iter()
        .filter(|function| function.name == CALL_METHOD)
    {
        lines.push(format!("{OVERLOAD} {}", typename(&call.into_type())));
    }
}

fn definition(
    lines: &mut Vec<String>,
    owner: &str,
    record: &RecordGenerator,
    function: &ExportedFunction,
) {
    if let Some(doc) = record.documentation.get(&function.name) {
        comment(lines, doc);
    }
    for param in &function.params {
        lines.push(parameter(param));
    }
    for returned in &function.returns {
        lines.push(format!("{RETURN} {}", typename(returned)));
    }

    let names: Vec<&str> = function.params.iter().map(param_name).collect();
    lines.push(format!(
        "{FUNCTION} {owner}.{}({}) {END}",
        function.name,
        names.join(", ")
    ));
}

fn parameter(param: &FunctionParam) -> String {
    if let Type::Variadic(inner) = &param.ty {
        return format!("{PARAM} {ELLIPSIS} {}", typename(inner));
    }

    let union = unwrapped(&param.ty);
    format!(
        "{PARAM} {}{} {}",
        param_name(param),
        suffix(union.optional),
        union.text
    )
}

fn param_name(param: &FunctionParam) -> &str {
    if matches!(param.ty, Type::Variadic(_)) {
        return ELLIPSIS;
    }

    param
        .param_name
        .as_ref()
        .map_or(UNNAMED, |name| name.0.as_ref())
}

fn typename(ty: &Type) -> String {
    match ty {
        Type::Single(single) => single.name.0.to_string(),
        Type::Array(inner) => format!("{}{ARRAY}", grouped(inner)),
        Type::Map(map) => format!("{MAP}<{}, {}>", typename(&map.key), typename(&map.value)),
        Type::Or(_) => union(ty),
        Type::Function(function) => {
            let params: Vec<String> = function.params.iter().map(argument).collect();
            let returns: Vec<String> = function.returns.iter().map(typename).collect();

            match returns.is_empty() {
                true => format!("{FUN}({})", params.join(", ")),
                false => format!("{FUN}({}): {}", params.join(", "), returns.join(", ")),
            }
        }
        Type::Tuple(parts) => parts.iter().map(typename).collect::<Vec<_>>().join(", "),
        Type::Variadic(inner) => format!("{} {ELLIPSIS}", typename(inner)),
    }
}

fn argument(param: &FunctionParam) -> String {
    if let Type::Variadic(inner) = &param.ty {
        return format!("{ELLIPSIS}: {}", typename(inner));
    }

    let union = unwrapped(&param.ty);
    format!(
        "{}{}: {}",
        param_name(param),
        suffix(union.optional),
        union.text
    )
}

fn union(ty: &Type) -> String {
    let union = unwrapped(ty);

    match (union.optional, union.count) {
        (false, _) => union.text,
        (true, 1) => format!("{}{OPTIONAL}", union.text),
        (true, _) => format!("{}|{NIL}", union.text),
    }
}

fn grouped(ty: &Type) -> String {
    let text = typename(ty);

    match text.contains(['|', '?']) || text.starts_with(FUN) {
        true => format!("({text})"),
        false => text,
    }
}

fn unwrapped(ty: &Type) -> Union {
    let mut flat = Vec::new();
    flatten(ty, &mut flat);

    let optional = flat.iter().any(|part| is_nil(part));
    let kept: Vec<&Type> = flat.into_iter().filter(|part| !is_nil(part)).collect();
    let many = kept.len() > 1;
    let parts: Vec<String> = kept.iter().map(|part| member(part, many)).collect();

    if parts.is_empty() {
        return Union {
            text: NIL.to_string(),
            optional: false,
            count: 0,
        };
    }

    Union {
        text: parts.join("|"),
        optional,
        count: kept.len(),
    }
}

fn flatten<'a>(ty: &'a Type, flat: &mut Vec<&'a Type>) {
    match ty {
        Type::Or(parts) => parts.iter().for_each(|part| flatten(part, flat)),
        other => flat.push(other),
    }
}

fn member(part: &Type, many: bool) -> String {
    let text = typename(part);

    match many && matches!(part, Type::Function(_)) {
        true => format!("({text})"),
        false => text,
    }
}

fn is_nil(ty: &Type) -> bool {
    ty.single().is_some_and(|single| single.name.0 == NIL)
}

fn suffix(optional: bool) -> &'static str {
    match optional {
        true => OPTIONAL,
        false => "",
    }
}

#[cfg(test)]
mod tests {
    use super::super::constants::DEFINITIONS;
    use super::*;
    use crate::lua::ld::walker;

    #[test]
    fn renders_the_committed_definitions() {
        assert!(
            render(&walker()) == DEFINITIONS,
            "meta/ld.lua is stale; run packaging/meta/update.sh"
        );
    }

    #[test]
    fn a_type_reads_as_the_server_expects() {
        use crate::lua::ld::{Kind, Param};

        let cases: [(Kind, &str); 7] = [
            (Kind::List(&Kind::String), "string[]"),
            (
                Kind::Function(
                    &[Param {
                        name: "options",
                        kind: Kind::Optional(&Kind::Named("t.Options")),
                    }],
                    &[],
                ),
                "fun(options?: t.Options)",
            ),
            (
                Kind::List(&Kind::Or(&[Kind::String, Kind::Integer])),
                "(string|integer)[]",
            ),
            (Kind::Optional(&Kind::String), "string?"),
            (
                Kind::Optional(&Kind::Or(&[Kind::String, Kind::False])),
                "string|false|nil",
            ),
            (
                Kind::Or(&[
                    Kind::Function(
                        &[Param {
                            name: "file",
                            kind: Kind::String,
                        }],
                        &[Kind::Optional(&Kind::String)],
                    ),
                    Kind::False,
                ]),
                "(fun(file: string): string?)|false",
            ),
            (
                Kind::Function(
                    &[Param {
                        name: "...",
                        kind: Kind::Variadic(&Kind::String),
                    }],
                    &[Kind::Variadic(&Kind::Optional(&Kind::String))],
                ),
                "fun(...: string): string? ...",
            ),
        ];

        for (kind, expected) in cases {
            assert_eq!(typename(&kind.to_type()), expected);
        }
    }
}
