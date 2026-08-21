use tealr::TypeWalker;

use super::super::super::signature::{Collect, Describe, record};
use super::constants::{
    CARD_DOC, CARD_FIELDS, CARD_TYPENAME, DOC, LIST_FIELDS, NAMESPACE_TYPENAME,
};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .instance(NAMESPACE_TYPENAME, DOC)
        .record(
            record(NAMESPACE_TYPENAME, DOC)
                .fields(&CARD_FIELDS)
                .fields(&LIST_FIELDS),
        )
        .record(record(CARD_TYPENAME, CARD_DOC).fields(&CARD_FIELDS))
}
