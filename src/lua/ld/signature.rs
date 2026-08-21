use tealr::{
    EnumGenerator, ExportedFunction, Field as Member, FunctionParam, FunctionRepresentation,
    GlobalInstance, KindOfType, Name, NameContainer, RecordGenerator, ToTypename, Type,
    TypeGenerator, TypeWalker,
};

use super::constants::{ANY, BOOLEAN, FALSE, INTEGER, NIL, NUMBER, STRING, TABLE};

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    String,
    Integer,
    Number,
    Boolean,
    Table,
    Any,
    False,
    Word(&'static str),
    Named(&'static str),
    List(&'static Kind),
    Or(&'static [Kind]),
    Function(&'static [Param], &'static [Kind]),
    Variadic(&'static Kind),
    Optional(&'static Kind),
}

#[derive(Debug, Clone, Copy)]
pub struct Param {
    pub name: &'static str,
    pub kind: Kind,
}

#[derive(Debug, Clone, Copy)]
pub struct Signature {
    pub name: &'static str,
    pub params: &'static [Param],
    pub returns: &'static [Kind],
    pub doc: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct Field {
    pub name: &'static str,
    pub kind: Kind,
    pub doc: &'static str,
}

struct Described;

impl ToTypename for Described {
    fn to_typename() -> Type {
        builtin(ANY)
    }
}

impl Kind {
    pub fn to_type(self) -> Type {
        match self {
            Kind::String => builtin(STRING),
            Kind::Integer => builtin(INTEGER),
            Kind::Number => builtin(NUMBER),
            Kind::Boolean => builtin(BOOLEAN),
            Kind::Table => builtin(TABLE),
            Kind::Any => builtin(ANY),
            Kind::False => builtin(FALSE),
            Kind::Word(word) => builtin(&format!("\"{word}\"")),
            Kind::Named(name) => named(name),
            Kind::List(item) => Type::Array(Box::new(item.to_type())),
            Kind::Or(kinds) => Type::Or(types(kinds)),
            Kind::Function(params, returns) => Type::Function(FunctionRepresentation {
                params: parameters(params),
                returns: types(returns),
            }),
            Kind::Variadic(item) => Type::Variadic(Box::new(item.to_type())),
            Kind::Optional(item) => optional(item.to_type()),
        }
    }
}

impl Signature {
    pub fn exported(&self) -> ExportedFunction {
        self.function(false)
    }

    pub fn meta(&self) -> ExportedFunction {
        self.function(true)
    }

    fn function(&self, is_meta_method: bool) -> ExportedFunction {
        ExportedFunction {
            name: container(self.name),
            params: parameters(self.params),
            returns: types(self.returns),
            is_meta_method,
        }
    }
}

pub trait Describe: Sized {
    fn fields(self, fields: &[Field]) -> Self;

    fn functions(self, signatures: &[Signature]) -> Self;

    fn call(self, signature: &Signature) -> Self;

    fn options(self, signatures: &[Signature]) -> Self;
}

impl Describe for RecordGenerator {
    fn fields(mut self, fields: &[Field]) -> Self {
        for field in fields {
            self.documentation
                .insert(container(field.name), field.doc.to_string());
            self.fields.push(Member {
                name: container(field.name),
                ty: field.kind.to_type(),
            });
        }

        self
    }

    fn functions(mut self, signatures: &[Signature]) -> Self {
        for signature in signatures {
            self.documentation
                .insert(container(signature.name), signature.doc.to_string());
            self.functions.push(signature.exported());
        }

        self
    }

    fn call(mut self, signature: &Signature) -> Self {
        self.documentation
            .insert(container(signature.name), signature.doc.to_string());
        self.meta_function.push(signature.meta());

        self
    }

    fn options(mut self, signatures: &[Signature]) -> Self {
        for signature in signatures {
            let Some(param) = signature.params.first() else {
                continue;
            };

            self.documentation
                .insert(container(signature.name), signature.doc.to_string());
            self.fields.push(Member {
                name: container(signature.name),
                ty: optional(param.kind.to_type()),
            });
        }

        self
    }
}

pub trait Collect: Sized {
    fn record(self, record: RecordGenerator) -> Self;

    fn namespace(
        self,
        name: &str,
        doc: &str,
        build: impl FnOnce(RecordGenerator) -> RecordGenerator,
    ) -> Self;

    fn choices<'a>(self, name: &str, doc: &str, names: impl IntoIterator<Item = &'a str>) -> Self;

    fn instance(self, name: &str, doc: &str) -> Self;
}

impl Collect for TypeWalker {
    fn record(mut self, record: RecordGenerator) -> Self {
        self.given_types
            .push(TypeGenerator::Record(Box::new(record)));
        self
    }

    fn namespace(
        self,
        name: &str,
        doc: &str,
        build: impl FnOnce(RecordGenerator) -> RecordGenerator,
    ) -> Self {
        self.instance(name, doc).record(build(record(name, doc)))
    }

    fn choices<'a>(
        mut self,
        name: &str,
        doc: &str,
        names: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        let mut generator = EnumGenerator::new::<Described>();
        generator.ty = named(name);
        generator.name = name.to_string();
        generator.variants = names.into_iter().map(container).collect();
        generator.type_doc = doc.to_string();

        self.given_types.push(TypeGenerator::Enum(generator));
        self
    }

    fn instance(mut self, name: &str, doc: &str) -> Self {
        self.global_instances_off.push(GlobalInstance {
            name: name.to_string(),
            ty: named(name),
            doc: doc.to_string(),
        });
        self
    }
}

pub fn record(name: &str, doc: &str) -> RecordGenerator {
    let mut record = RecordGenerator::new::<Described>(false);
    record.ty = named(name);
    record.type_doc = doc.to_string();
    record.should_generate_help_method = false;

    record
}

pub fn optional(ty: Type) -> Type {
    Type::Or(vec![ty, builtin(NIL)])
}

pub fn named(name: &str) -> Type {
    Type::new_single(name, KindOfType::External)
}

fn builtin(name: &str) -> Type {
    Type::new_single(name, KindOfType::Builtin)
}

fn container(name: &str) -> NameContainer {
    NameContainer::from(name.to_string())
}

fn types(kinds: &[Kind]) -> Vec<Type> {
    kinds.iter().map(|kind| kind.to_type()).collect()
}

fn parameters(params: &[Param]) -> Vec<FunctionParam> {
    params
        .iter()
        .map(|param| FunctionParam {
            param_name: Some(Name::from(param.name)),
            ty: param.kind.to_type(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_optional_kind_is_a_union_with_nil() {
        let Type::Or(parts) = Kind::Optional(&Kind::String).to_type() else {
            panic!("not a union");
        };

        assert_eq!(parts, [builtin(STRING), builtin(NIL)]);
    }

    #[test]
    fn a_word_is_a_quoted_literal() {
        assert_eq!(
            Kind::Word("passphrase").to_type(),
            builtin("\"passphrase\"")
        );
    }

    #[test]
    fn the_options_of_a_record_are_its_setters_made_optional() {
        const SET: [Signature; 1] = [Signature {
            name: "link",
            params: &[Param {
                name: "mode",
                kind: Kind::Named("ld.LinkMode"),
            }],
            returns: &[],
            doc: "How files are placed.",
        }];

        let record = record("ld.Options", "").options(&SET);

        assert_eq!(record.fields.len(), 1);
        assert_eq!(record.fields[0].name, "link");
        assert_eq!(record.fields[0].ty, optional(named("ld.LinkMode")));
        assert_eq!(
            record.documentation[&container("link")],
            "How files are placed."
        );
    }

    #[test]
    fn a_call_is_the_meta_function_of_the_record() {
        const CALL: Signature = Signature {
            name: "__call",
            params: &[],
            returns: &[],
            doc: "",
        };

        let record = record("ld.opt", "").call(&CALL);

        assert!(record.functions.is_empty());
        assert!(record.meta_function[0].is_meta_method);
        assert_eq!(record.meta_function[0].name, "__call");
    }
}
