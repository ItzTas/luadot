use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, record};
use super::constants::{DOC, NAMESPACE_TYPENAME, SIGNATURES};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .instance(NAMESPACE_TYPENAME, DOC)
        .record(record(NAMESPACE_TYPENAME, DOC).functions(&SIGNATURES))
}
