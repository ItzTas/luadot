use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, record};
use super::constants::{
    CALL, DOC, IDENTITY_DOC, IDENTITY_FIELDS, IDENTITY_KIND_DOC, IDENTITY_KIND_TYPENAME,
    IDENTITY_TYPENAME, IDENTITY_TYPES, KEYS_DOC, KEYS_FIELDS, KEYS_TYPENAME, NAMESPACE_TYPENAME,
    OPTIONS_DOC, OPTIONS_TYPENAME, SIGNATURES,
};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .choices(
            IDENTITY_KIND_TYPENAME,
            IDENTITY_KIND_DOC,
            IDENTITY_TYPES.iter().map(|(name, _)| *name),
        )
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES).call(&CALL)
        })
        .record(record(OPTIONS_TYPENAME, OPTIONS_DOC).options(&SIGNATURES))
        .record(record(KEYS_TYPENAME, KEYS_DOC).fields(&KEYS_FIELDS))
        .record(record(IDENTITY_TYPENAME, IDENTITY_DOC).fields(&IDENTITY_FIELDS))
}
