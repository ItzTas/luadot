use tealr::TypeWalker;

use super::super::signature::{Collect, Describe, Field, Kind, record};
use super::constants::{
    CONTENT_DOC, CONTENT_TYPENAME, COUNT_DOC, COUNTS_FIELDS, DIFF_COUNTS_DOC, DIFF_COUNTS_FIELDS,
    DIFF_COUNTS_TYPENAME, DIFF_FIELDS, DIFF_FILE_DOC, DIFF_FILE_FIELDS, DIFF_FILE_TYPENAME,
    DIFF_OPTIONS_DOC, DIFF_OPTIONS_TYPENAME, DIFF_STATE_DOC, DIFF_STATE_TYPENAME, DOC, FILE_FIELDS,
    MODE_DOC, MODE_TYPENAME, NAMESPACE_TYPENAME, SIDE_DOC, SIDE_FIELDS, SIDE_TYPENAME, SIDES,
    SIGNATURES, STATUS_COUNTS_DOC, STATUS_COUNTS_FIELDS, STATUS_COUNTS_TYPENAME, STATUS_FIELDS,
    STATUS_FILE_DOC, STATUS_FILE_FIELDS, STATUS_FILE_TYPENAME, STATUS_OPTIONS_DOC,
    STATUS_OPTIONS_TYPENAME, STATUS_STATE_DOC, STATUS_STATE_TYPENAME, STATUS_STATES,
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
        .instance(NAMESPACE_TYPENAME, DOC)
        .record(record(NAMESPACE_TYPENAME, DOC).functions(&SIGNATURES))
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

#[cfg(test)]
mod tests {
    use super::super::constants::{DIFF_KEYS, FUNCTIONS, REPORT_KEYS};
    use super::*;

    #[test]
    fn every_function_is_described_in_the_order_it_is_registered() {
        let registered: Vec<&str> = FUNCTIONS.iter().map(|(name, _)| *name).collect();
        let described: Vec<&str> = SIGNATURES.iter().map(|signature| signature.name).collect();

        assert_eq!(described, registered);
    }

    #[test]
    fn every_key_of_each_command_is_described_in_the_order_it_is_read() {
        let diff: Vec<&str> = DIFF_FIELDS.iter().map(|field| field.name).collect();
        let status: Vec<&str> = STATUS_FIELDS.iter().map(|field| field.name).collect();

        assert_eq!(diff, DIFF_KEYS);
        assert_eq!(status, REPORT_KEYS);
    }
}
