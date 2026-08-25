use tealr::TypeWalker;

use super::super::signature::{Collect, Describe};
use super::constants::{DOC, FIELDS, NAMESPACE_TYPENAME, SIGNATURES};
use super::{gpu, host};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    let walker = walker.namespace(NAMESPACE_TYPENAME, DOC, |record| {
        record.fields(&FIELDS).functions(&SIGNATURES)
    });

    host::describe(gpu::describe(walker))
}
