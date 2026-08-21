use tealr::TypeWalker;

use super::super::constants::API;
use super::super::signature::{Collect, Describe, record};
use super::constants::{API_DOC, FIELDS, RULE_DOC, RULE_FIELDS, RULE_TYPENAME, SIGNATURES};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .namespace(API, API_DOC, |record| {
            record.fields(&FIELDS).functions(&SIGNATURES)
        })
        .record(record(RULE_TYPENAME, RULE_DOC).fields(&RULE_FIELDS))
}
