use tealr::TypeWalker;

use super::constants::{
    BACKEND_DOC, BACKEND_TYPENAME, CONFLICT_DOC, CONFLICT_POLICIES, CONFLICT_TYPENAME,
    CRYPT_BACKENDS, LINK_MODE_DOC, LINK_MODE_TYPENAME, LINK_MODES,
};
use super::signature::Collect;
use super::{
    alt, argv, class, cmd, crypt, fs, git, json, on, opt, path, pkg, print, regex, root, rtp,
    setup, sys,
};

type Describer = fn(TypeWalker) -> TypeWalker;

pub fn walker() -> TypeWalker {
    let describers: [Describer; 18] = [
        root::describe,
        alt::describe,
        argv::describe,
        class::describe,
        cmd::describe,
        crypt::describe,
        fs::describe,
        git::describe,
        json::describe,
        on::describe,
        opt::describe,
        path::describe,
        pkg::describe,
        print::describe,
        regex::describe,
        rtp::describe,
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
    use tealr::{KindOfType, RecordGenerator, Type, TypeGenerator};

    use super::super::constants::{
        API, BOOLEAN, CALL_METHOD, INTEGER, INTEGER_INDEX, LIGHT_USERDATA, NUMBER, STRING,
        STRING_INDEX,
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
        let paths = Paths::new(
            Path::new("/home/u"),
            Path::new("/home/u/.config/luadot"),
            Path::new("/home/u/.local/share/luadot"),
        )
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

    fn shape_of_type(ty: &Type, aliases: &BTreeSet<String>) -> (Shape, bool) {
        let plain = Shape::Table { callable: false };

        match ty {
            Type::Or(parts) => {
                let optional = parts.iter().any(is_nil);
                let shape = parts
                    .iter()
                    .find(|part| !is_nil(part))
                    .map_or(plain, |part| shape_of_type(part, aliases).0);

                (shape, optional)
            }
            Type::Function(_) => (Shape::Function, false),
            Type::Single(single) => match single.name.0.as_ref() {
                STRING | BOOLEAN | LIGHT_USERDATA => {
                    (Shape::Value(single.name.0.to_string()), false)
                }
                INTEGER | NUMBER => (Shape::Value(NUMBER.to_string()), false),
                name if aliases.contains(name) => (Shape::Value(STRING.to_string()), false),
                _ => (plain, false),
            },
            _ => (plain, false),
        }
    }

    fn aliases(walker: &TypeWalker) -> BTreeSet<String> {
        walker
            .given_types
            .iter()
            .filter_map(|generator| match generator {
                TypeGenerator::Enum(choices) => Some(choices.name.clone()),
                TypeGenerator::Record(_) => None,
            })
            .collect()
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
        let aliases = aliases(walker);

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
                    shape_of_type(&field.ty, &aliases),
                );
            }
        }

        found
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
}
