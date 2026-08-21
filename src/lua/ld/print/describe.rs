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
