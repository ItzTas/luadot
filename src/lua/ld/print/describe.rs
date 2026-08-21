use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, record};
use super::constants::{
    CALL, COLOR_DOC, COLOR_TYPENAME, COLORS, DOC, NAMESPACE_TYPENAME, OPTIONS_DOC, OPTIONS_FIELDS,
    OPTIONS_TYPENAME, SIGNATURES, STREAM_DOC, STREAM_TYPENAME, STREAMS, TONE_DOC, TONE_TYPENAME,
    TONES,
};

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .choices(TONE_TYPENAME, TONE_DOC, TONES.iter().map(|(name, _)| *name))
        .choices(
            STREAM_TYPENAME,
            STREAM_DOC,
            STREAMS.iter().map(|(name, _)| *name),
        )
        .choices(
            COLOR_TYPENAME,
            COLOR_DOC,
            COLORS.iter().map(|(name, _)| *name),
        )
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES).call(&CALL)
        })
        .record(record(OPTIONS_TYPENAME, OPTIONS_DOC).fields(&OPTIONS_FIELDS))
}

#[cfg(test)]
mod tests {
    use super::super::constants::{FUNCTIONS, OPTIONS};
    use super::*;

    #[test]
    fn every_function_is_described_in_the_order_it_is_registered() {
        let registered: Vec<&str> = FUNCTIONS.iter().map(|(name, _)| *name).collect();
        let described: Vec<&str> = SIGNATURES.iter().map(|signature| signature.name).collect();

        assert_eq!(described, registered);
    }

    #[test]
    fn every_option_is_described_in_the_order_it_is_read() {
        let described: Vec<&str> = OPTIONS_FIELDS.iter().map(|field| field.name).collect();

        assert_eq!(described, OPTIONS);
    }
}
