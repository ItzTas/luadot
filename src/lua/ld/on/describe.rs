use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, Field, Kind, record};
use super::constants::{
    AROUND_DOC, AROUND_FIELDS, AROUND_TYPENAME, CONTENT_DOC, CONTENT_TYPENAME, COUNT_DOC,
    COUNTS_FIELDS, DIFF_COUNTS_DOC, DIFF_COUNTS_FIELDS, DIFF_COUNTS_TYPENAME, DIFF_FIELDS,
    DIFF_FILE_DOC, DIFF_FILE_FIELDS, DIFF_FILE_TYPENAME, DIFF_OPTIONS_DOC, DIFF_OPTIONS_TYPENAME,
    DIFF_STATE_DOC, DIFF_STATE_TYPENAME, DOC, FILE_FIELDS, MODE_DOC, MODE_TYPENAME,
    NAMESPACE_TYPENAME, SIDE_DOC, SIDE_FIELDS, SIDE_TYPENAME, SIDES, SIGNATURES, STATUS_COUNTS_DOC,
    STATUS_COUNTS_FIELDS, STATUS_COUNTS_TYPENAME, STATUS_FIELDS, STATUS_FILE_DOC,
    STATUS_FILE_FIELDS, STATUS_FILE_TYPENAME, STATUS_OPTIONS_DOC, STATUS_OPTIONS_TYPENAME,
    STATUS_STATE_DOC, STATUS_STATE_TYPENAME, STATUS_STATES, TMPL_DOC, TMPL_SIGNATURES,
    TMPL_TYPENAME,
};
use crate::lua::config::constants::DIFF_STATES;

pub fn describe(walker: TypeWalker) -> TypeWalker {
    walker
        .choices(SIDE_TYPENAME, SIDE_DOC, SIDES.iter().map(|side| side.dir()))
        .choices(
            DIFF_STATE_TYPENAME,
            DIFF_STATE_DOC,
            DIFF_STATES.iter().map(|(name, _)| *name),
        )
        .choices(
            STATUS_STATE_TYPENAME,
            STATUS_STATE_DOC,
            STATUS_STATES.iter().map(|state| state.name()),
        )
        .namespace(NAMESPACE_TYPENAME, DOC, |record| {
            record.functions(&SIGNATURES)
        })
        .namespace(TMPL_TYPENAME, TMPL_DOC, |record| {
            record.functions(&TMPL_SIGNATURES)
        })
        .record(record(AROUND_TYPENAME, AROUND_DOC).fields(&AROUND_FIELDS))
        .record(record(DIFF_OPTIONS_TYPENAME, DIFF_OPTIONS_DOC).fields(&DIFF_FIELDS))
        .record(record(STATUS_OPTIONS_TYPENAME, STATUS_OPTIONS_DOC).fields(&STATUS_FIELDS))
        .record(
            record(DIFF_FILE_TYPENAME, DIFF_FILE_DOC)
                .fields(&FILE_FIELDS)
                .fields(&DIFF_FILE_FIELDS),
        )
        .record(
            record(STATUS_FILE_TYPENAME, STATUS_FILE_DOC)
                .fields(&FILE_FIELDS)
                .fields(&STATUS_FILE_FIELDS),
        )
        .record(record(CONTENT_TYPENAME, CONTENT_DOC).fields(&SIDE_FIELDS))
        .record(record(MODE_TYPENAME, MODE_DOC).fields(&SIDE_FIELDS))
        .record(
            record(DIFF_COUNTS_TYPENAME, DIFF_COUNTS_DOC)
                .fields(&COUNTS_FIELDS)
                .fields(&DIFF_COUNTS_FIELDS),
        )
        .record(
            record(STATUS_COUNTS_TYPENAME, STATUS_COUNTS_DOC)
                .fields(&COUNTS_FIELDS)
                .fields(&STATUS_COUNTS_FIELDS)
                .fields(&counts()),
        )
}

fn counts() -> Vec<Field> {
    STATUS_STATES
        .iter()
        .map(|state| Field {
            name: state.name(),
            kind: Kind::Integer,
            doc: COUNT_DOC,
        })
        .collect()
}
