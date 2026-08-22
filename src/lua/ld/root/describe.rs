use tealr::TypeWalker;

use super::super::constants::API;
use super::super::signature::{Collect, Describe, record};
use super::super::surface::{SURFACES, Surface};
use super::constants::{
    API_DOC, FIELDS, RULE_DOC, RULE_FIELDS, RULE_TYPENAME, SIGNATURES, SURFACE_DOC,
    SURFACE_TYPENAME, TASK_DOC, TASK_FIELDS, TASK_TYPENAME,
};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .choices(SURFACE_TYPENAME, SURFACE_DOC, SURFACES.map(Surface::name))
        .namespace(API, API_DOC, |record| {
            record.fields(&FIELDS).functions(&SIGNATURES)
        })
        .record(record(RULE_TYPENAME, RULE_DOC).fields(&RULE_FIELDS))
        .record(record(TASK_TYPENAME, TASK_DOC).fields(&TASK_FIELDS))
}

#[cfg(test)]
mod tests {
    use super::super::constants::TASK_KEYS;
    use super::*;

    #[test]
    fn every_task_key_is_described_in_the_order_it_is_read() {
        let described: Vec<&str> = TASK_FIELDS.iter().map(|field| field.name).collect();

        assert_eq!(described, TASK_KEYS);
    }
}
