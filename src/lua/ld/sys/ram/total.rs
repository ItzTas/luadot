use std::fs;
use std::path::Path;

use super::constants::{KIB, MEMINFO_FILE, TOTAL_FIELD};

pub fn total() -> u64 {
    read(Path::new(MEMINFO_FILE))
}

fn read(path: &Path) -> u64 {
    fs::read_to_string(path)
        .ok()
        .and_then(|meminfo| parse(&meminfo))
        .unwrap_or_default()
}

fn parse(meminfo: &str) -> Option<u64> {
    meminfo
        .lines()
        .find_map(|line| line.trim().strip_prefix(TOTAL_FIELD))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
        .map(|kib| kib * KIB)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEMINFO: &str = "MemTotal:       32723660 kB\nMemFree:         1234 kB\n";

    #[test]
    fn reads_the_total_in_bytes() {
        assert_eq!(parse(MEMINFO), Some(33_509_027_840));
    }

    #[test]
    fn a_file_without_the_field_reports_nothing() {
        assert_eq!(parse("MemFree: 1234 kB\n"), None);
        assert_eq!(parse("MemTotal: what kB\n"), None);
    }
}
