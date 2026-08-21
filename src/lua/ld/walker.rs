use tealr::TypeWalker;

use super::constants::{
    BACKEND_DOC, BACKEND_TYPENAME, CONFLICT_DOC, CONFLICT_POLICIES, CONFLICT_TYPENAME,
    CRYPT_BACKENDS, LINK_MODE_DOC, LINK_MODE_TYPENAME, LINK_MODES,
};
use super::signature::Collect;
use super::{
    alt, argv, class, cmd, crypt, git, on, opt, path, pkg, print, regex, root, setup, sys,
};

type Describer = fn(TypeWalker) -> TypeWalker;

pub fn walker() -> TypeWalker {
    let describers: [Describer; 15] = [
        root::describe,
        alt::describe,
        argv::describe,
        class::describe,
        cmd::describe,
        crypt::describe,
        git::describe,
        on::describe,
        opt::describe,
        path::describe,
        pkg::describe,
        print::describe,
        regex::describe,
        setup::describe,
        sys::describe,
    ];

    let walker = TypeWalker::new()
        .choices(
            LINK_MODE_TYPENAME,
            LINK_MODE_DOC,
            LINK_MODES.iter().map(|(name, _)| *name),
        )
        .choices(
            CONFLICT_TYPENAME,
            CONFLICT_DOC,
            CONFLICT_POLICIES.iter().map(|(name, _)| *name),
        )
        .choices(
            BACKEND_TYPENAME,
            BACKEND_DOC,
            CRYPT_BACKENDS.iter().map(|(name, _)| *name),
        );

    describers
        .into_iter()
        .fold(walker, |walker, describe| describe(walker))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;

    use mlua::{Table, Value};
    use tealr::{ExportedFunction, KindOfType, RecordGenerator, Type, TypeGenerator};

    use super::super::constants::{
        API, BOOLEAN, CALL_METHOD, INTEGER, INTEGER_INDEX, NUMBER, STRING, STRING_INDEX,
    };
    use super::super::{Paths, Surface, install};
    use super::*;
    use crate::lua::runtime::runtime;
    use crate::state::Classes;

    #[derive(Debug, PartialEq, Eq)]
    enum Shape {
        Function,
        Table { callable: bool },
        Value(String),
    }

    fn installed() -> Table {
        let lua = runtime().unwrap();
        let paths = Paths::new(Path::new("/home/u"), Path::new("/home/u/.config/luadot"))
            .with_repo(Some(Path::new("/data/repo")));
        install(&lua, Surface::Bootstrap, &paths, &Classes::default()).unwrap();

        let ld: Table = lua.globals().get(API).unwrap();
        std::mem::forget(lua);
        ld
    }

    fn callable(table: &Table) -> bool {
        table
            .metatable()
            .and_then(|meta| meta.get::<Value>(CALL_METHOD).ok())
            .is_some_and(|call| call.is_function())
    }

    fn shape_of_value(value: &Value) -> Shape {
        match value {
            Value::Function(_) => Shape::Function,
            Value::Table(table) => Shape::Table {
                callable: callable(table),
            },
            Value::Integer(_) | Value::Number(_) => Shape::Value(NUMBER.to_string()),
            other => Shape::Value(other.type_name().to_string()),
        }
    }

    fn walk(prefix: &str, table: &Table, found: &mut BTreeMap<String, Shape>) {
        for pair in table.clone().pairs::<Value, Value>() {
            let (key, value) = pair.unwrap();
            let Value::String(key) = key else {
                continue;
            };

            let path = format!("{prefix}.{}", key.to_str().unwrap());
            if let Value::Table(inner) = &value {
                walk(&path, inner, found);
            }
            found.insert(path, shape_of_value(&value));
        }
    }

    fn live(ld: &Table) -> BTreeMap<String, Shape> {
        let mut found = BTreeMap::new();
        found.insert(API.to_string(), shape_of_value(&Value::Table(ld.clone())));
        walk(API, ld, &mut found);

        found
    }

    fn reached(ld: &Table, path: &str) -> Value {
        path.split('.')
            .skip(1)
            .fold(Value::Table(ld.clone()), |value, name| match value {
                Value::Table(table) => table.get(name).unwrap(),
                _ => Value::Nil,
            })
    }

    fn is_nil(ty: &Type) -> bool {
        ty.single()
            .is_some_and(|single| single.name.0 == "nil" && single.kind == KindOfType::Builtin)
    }

    fn shape_of_type(ty: &Type) -> (Shape, bool) {
        let plain = Shape::Table { callable: false };

        match ty {
            Type::Or(parts) => {
                let optional = parts.iter().any(is_nil);
                let shape = parts
                    .iter()
                    .find(|part| !is_nil(part))
                    .map_or(plain, |part| shape_of_type(part).0);

                (shape, optional)
            }
            Type::Function(_) => (Shape::Function, false),
            Type::Single(single) => match single.name.0.as_ref() {
                STRING | BOOLEAN => (Shape::Value(single.name.0.to_string()), false),
                INTEGER | NUMBER => (Shape::Value(NUMBER.to_string()), false),
                _ => (plain, false),
            },
            _ => (plain, false),
        }
    }

    fn record<'a>(walker: &'a TypeWalker, ty: &Type) -> &'a RecordGenerator {
        walker
            .given_types
            .iter()
            .find_map(|generator| match generator {
                TypeGenerator::Record(record) if &record.ty == ty => Some(record.as_ref()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{ty:?} has no record"))
    }

    fn described(walker: &TypeWalker) -> BTreeMap<String, (Shape, bool)> {
        let mut found = BTreeMap::new();

        for instance in &walker.global_instances_off {
            let record = record(walker, &instance.ty);
            let callable = record.meta_function.iter().any(|f| f.name == CALL_METHOD);
            found.insert(instance.name.clone(), (Shape::Table { callable }, false));

            for function in &record.functions {
                found.insert(
                    format!("{}.{}", instance.name, function.name),
                    (Shape::Function, false),
                );
            }
            for field in &record.fields {
                if field.name == STRING_INDEX || field.name == INTEGER_INDEX {
                    continue;
                }
                found.insert(
                    format!("{}.{}", instance.name, field.name),
                    shape_of_type(&field.ty),
                );
            }
        }

        found
    }

    fn typenames(walker: &TypeWalker) -> BTreeSet<String> {
        walker
            .given_types
            .iter()
            .filter_map(|generator| generator.type_name().single())
            .map(|single| single.name.0.to_string())
            .collect()
    }

    fn mentioned(ty: &Type, found: &mut BTreeSet<String>) {
        match ty {
            Type::Single(single) if single.kind == KindOfType::External => {
                found.insert(single.name.0.to_string());
            }
            Type::Single(_) => {}
            Type::Function(function) => {
                for param in &function.params {
                    mentioned(&param.ty, found);
                }
                for ret in &function.returns {
                    mentioned(ret, found);
                }
            }
            Type::Map(map) => {
                mentioned(&map.key, found);
                mentioned(&map.value, found);
            }
            Type::Or(parts) | Type::Tuple(parts) => {
                for part in parts {
                    mentioned(part, found);
                }
            }
            Type::Array(inner) | Type::Variadic(inner) => mentioned(inner, found),
        }
    }

    fn mentioned_by(function: &ExportedFunction, found: &mut BTreeSet<String>) {
        for param in &function.params {
            mentioned(&param.ty, found);
        }
        for ret in &function.returns {
            mentioned(ret, found);
        }
    }

    #[test]
    fn the_description_and_the_installed_interface_carry_the_same_names_kinds_and_calls() {
        let walker = walker();
        let ld = installed();
        let described = described(&walker);
        let live = live(&ld);

        for (path, (shape, optional)) in &described {
            match live.get(path) {
                Some(found) => assert_eq!(found, shape, "{path}"),
                None if *optional => {}
                None => assert_eq!(
                    shape_of_value(&reached(&ld, path)),
                    *shape,
                    "{path} is described but not installed"
                ),
            }
        }
        for path in live.keys() {
            assert!(
                described.contains_key(path),
                "{path} is installed but not described"
            );
        }
    }

    #[test]
    fn a_namespace_indexed_by_a_program_name_says_so() {
        let walker = walker();
        let ld = installed();

        for instance in &walker.global_instances_off {
            if instance.name == API {
                continue;
            }

            let indexed = record(&walker, &instance.ty)
                .fields
                .iter()
                .any(|field| field.name == STRING_INDEX);
            let Value::Table(table) = reached(&ld, &instance.name) else {
                panic!("{} is not a table", instance.name);
            };
            let resolver = table
                .metatable()
                .and_then(|meta| meta.get::<Value>("__index").ok())
                .is_some_and(|index| index.is_function());

            assert_eq!(resolver, indexed, "{}", instance.name);
        }
    }

    #[test]
    fn every_type_a_description_names_is_declared() {
        let walker = walker();
        let declared = typenames(&walker);
        let mut named = BTreeSet::new();

        for instance in &walker.global_instances_off {
            mentioned(&instance.ty, &mut named);
        }
        for generator in &walker.given_types {
            let TypeGenerator::Record(record) = generator else {
                continue;
            };
            for field in &record.fields {
                mentioned(&field.ty, &mut named);
            }
            for function in record.all_functions() {
                mentioned_by(function, &mut named);
            }
        }

        let missing: Vec<&String> = named.difference(&declared).collect();
        assert!(missing.is_empty(), "undeclared: {missing:?}");
    }

    #[test]
    fn every_value_the_parser_takes_reaches_its_alias() {
        let walker = walker();
        let expected: [(&str, Vec<&str>); 3] = [
            (
                LINK_MODE_TYPENAME,
                LINK_MODES.iter().map(|(name, _)| *name).collect(),
            ),
            (
                CONFLICT_TYPENAME,
                CONFLICT_POLICIES.iter().map(|(name, _)| *name).collect(),
            ),
            (
                BACKEND_TYPENAME,
                CRYPT_BACKENDS.iter().map(|(name, _)| *name).collect(),
            ),
        ];

        for (name, variants) in expected {
            let found = walker
                .given_types
                .iter()
                .find_map(|generator| match generator {
                    TypeGenerator::Enum(choices) if choices.name == name => Some(choices),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{name} is not declared"));
            let found: Vec<String> = found.variants.iter().map(ToString::to_string).collect();

            assert_eq!(found, variants, "{name}");
        }
    }
}
