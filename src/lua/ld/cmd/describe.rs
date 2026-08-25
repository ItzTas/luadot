use tealr::TypeWalker;

use super::super::signature::{Collect, Describe};
use super::constants::{CALL, DOC, FIELDS, NAMESPACE_TYPENAME};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker.namespace(NAMESPACE_TYPENAME, DOC, |record| {
        record.fields(&FIELDS).call(&CALL)
    })
}
