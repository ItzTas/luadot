use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, record};
use super::constants::{
    CALL, CLASS_DOC, CLASS_FIELDS, CLASS_TYPENAME, DOC, NAMESPACE_TYPENAME, SIGNATURES,
};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .instance(NAMESPACE_TYPENAME, DOC)
        .record(
            record(NAMESPACE_TYPENAME, DOC)
                .functions(&SIGNATURES)
                .call(&CALL),
        )
        .record(record(CLASS_TYPENAME, CLASS_DOC).fields(&CLASS_FIELDS))
}
