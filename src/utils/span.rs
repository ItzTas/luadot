const SPAN_UNITS: [(u64, &str); 4] = [
    (86_400, "day"),
    (3_600, "hour"),
    (60, "minute"),
    (1, "second"),
];

const SPAN_SUFFIXES: [(&str, u64); 5] = [
    ("s", 1),
    ("m", 60),
    ("h", 3_600),
    ("d", 86_400),
    ("w", 604_800),
];

pub fn seconds(raw: &str) -> Option<u64> {
    let trimmed = raw.trim();
    let split = trimmed.find(|digit: char| !digit.is_ascii_digit())?;
    let (count, suffix) = trimmed.split_at(split);

    let count: u64 = count.parse().ok()?;
    let (_, size) = SPAN_SUFFIXES.iter().find(|(name, _)| *name == suffix)?;

    count.checked_mul(*size)
}

pub fn span(seconds: u64) -> String {
    for (size, name) in SPAN_UNITS {
        if seconds < size {
            continue;
        }
        let count = seconds / size;
        return format!("{count} {name}{}", plural(count));
    }

    format!("{seconds} seconds")
}

fn plural(count: u64) -> &'static str {
    match count {
        1 => "",
        _ => "s",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_suffix_is_read_as_its_own_unit() {
        assert_eq!(seconds("45s"), Some(45));
        assert_eq!(seconds("90m"), Some(5_400));
        assert_eq!(seconds("12h"), Some(43_200));
        assert_eq!(seconds("30d"), Some(2_592_000));
        assert_eq!(seconds("2w"), Some(1_209_600));
    }

    #[test]
    fn a_span_uses_the_largest_unit() {
        assert_eq!(span(1), "1 second");
        assert_eq!(span(59), "59 seconds");
        assert_eq!(span(60), "1 minute");
        assert_eq!(span(3_600), "1 hour");
        assert_eq!(span(7_200), "2 hours");
        assert_eq!(span(86_400), "1 day");
        assert_eq!(span(1_209_600), "14 days");
        assert_eq!(span(0), "0 seconds");
    }
}
