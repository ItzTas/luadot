use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, record};
use super::constants::{DOC, FIELDS, NAMESPACE_TYPENAME, SIGNATURES};
use super::{gpu, host};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    let walker = walker.instance(NAMESPACE_TYPENAME, DOC).record(
        record(NAMESPACE_TYPENAME, DOC)
            .fields(&FIELDS)
            .functions(&SIGNATURES),
    );

    host::describe(gpu::describe(walker))
}
