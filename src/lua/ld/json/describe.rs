use tealr::TypeWalker;

use super::super::signature::{Collect, Describe};
use super::constants::{DOC, FIELDS, NAMESPACE_TYPENAME, SIGNATURES};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker.namespace(NAMESPACE_TYPENAME, DOC, |record| {
        record.fields(&FIELDS).functions(&SIGNATURES)
    })
}
