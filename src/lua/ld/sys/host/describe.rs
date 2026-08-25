use tealr::TypeWalker;

use super::super::super::signature::{Collect, Describe};
use super::constants::{DOC, FIELDS, NAMESPACE_TYPENAME};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker.namespace(NAMESPACE_TYPENAME, DOC, |record| record.fields(&FIELDS))
}
