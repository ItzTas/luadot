use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, record};
use super::constants::{
    CALL, CLONE_FIELDS, CLONE_OPTIONS_DOC, CLONE_OPTIONS_TYPENAME, DOC, NAMESPACE_TYPENAME,
    SIGNATURES,
};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES).call(&CALL)
        })
        .record(record(CLONE_OPTIONS_TYPENAME, CLONE_OPTIONS_DOC).fields(&CLONE_FIELDS))
}

#[cfg(test)]
mod tests {
    use super::super::constants::CLONE_KEYS;
    use super::*;

    #[test]
    fn every_clone_key_is_described() {
        let described: Vec<&str> = CLONE_FIELDS.iter().map(|field| field.name).collect();

        assert_eq!(described, CLONE_KEYS);
    }
}
