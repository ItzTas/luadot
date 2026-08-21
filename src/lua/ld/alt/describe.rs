use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, record};
use super::constants::{
    DOC, FILE_DOC, FILE_TYPENAME, NAMESPACE_TYPENAME, OUTPUT_DOC, OUTPUT_FIELDS, OUTPUT_TYPENAME,
    SIGNATURES,
};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .instance(NAMESPACE_TYPENAME, DOC)
        .record(record(NAMESPACE_TYPENAME, DOC).functions(&SIGNATURES))
        .record(record(OUTPUT_TYPENAME, OUTPUT_DOC).fields(&OUTPUT_FIELDS))
        .record(record(FILE_TYPENAME, FILE_DOC))
}
