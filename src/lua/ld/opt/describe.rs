use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, record};
use super::constants::{CALL, DOC, NAMESPACE_TYPENAME, OPTIONS_DOC, OPTIONS_TYPENAME, SIGNATURES};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES).call(&CALL)
        })
        .record(record(OPTIONS_TYPENAME, OPTIONS_DOC).options(&SIGNATURES))
}
