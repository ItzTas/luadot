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

#[cfg(test)]
mod tests {
    use super::super::constants::RULE_KEYS;
    use super::*;

    #[test]
    fn every_rule_key_is_described_in_the_order_it_is_read() {
        let described: Vec<&str> = RULE_FIELDS.iter().map(|field| field.name).collect();

        assert_eq!(described, RULE_KEYS);
    }
}
