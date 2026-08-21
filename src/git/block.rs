pub(super) fn stitched(current: &str, start: &str, end: &str, body: &str) -> String {
    let kept = outside(current, start, end);
    let block = match body.trim().is_empty() {
        true => String::new(),
        false => format!("{start}\n{}\n{end}\n", body.trim_end()),
    };

    match (kept.is_empty(), block.is_empty()) {
        (true, true) => String::new(),
        (true, false) => block,
        (false, true) => format!("{kept}\n"),
        (false, false) => format!("{kept}\n\n{block}"),
    }
}

fn outside(current: &str, start: &str, end: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut inside = false;

    for line in current.lines() {
        if line.trim() == start {
            inside = true;
            continue;
        }
        if line.trim() == end {
            inside = false;
            continue;
        }
        if !inside {
            kept.push(line);
        }
    }

    while kept.last().is_some_and(|line| line.trim().is_empty()) {
        kept.pop();
    }

    kept.join("\n")
}
