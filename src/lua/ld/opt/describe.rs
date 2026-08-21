use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, record};
use super::constants::{CALL, DOC, NAMESPACE_TYPENAME, OPTIONS_DOC, OPTIONS_TYPENAME, SIGNATURES};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .instance(NAMESPACE_TYPENAME, DOC)
        .record(
            record(NAMESPACE_TYPENAME, DOC)
                .functions(&SIGNATURES)
                .call(&CALL),
        )
        .record(record(OPTIONS_TYPENAME, OPTIONS_DOC).options(&SIGNATURES))
}

#[cfg(test)]
mod tests {
    use super::super::constants::SETTERS;
    use super::*;

    #[test]
    fn every_setter_is_described_in_the_order_it_is_registered() {
        let registered: Vec<&str> = SETTERS.iter().map(|(name, _)| *name).collect();
        let described: Vec<&str> = SIGNATURES.iter().map(|signature| signature.name).collect();

        assert_eq!(described, registered);
    }
}
