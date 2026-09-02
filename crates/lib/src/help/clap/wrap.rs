//! Column layout and wrapping for the clap dialects

/// Two-space margin, labels padded to the widest, two-space gutter, description.
pub fn push_entries(out: &mut String, entries: &[(String, String)]) {
    let width = entries
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    let room = TERM_WIDTH.saturating_sub(width.saturating_add(4)).max(1);
    for (label, desc) in entries {
        let wrapped = wrap(desc, room);
        for (index, line) in wrapped.iter().enumerate() {
            if index == 0 {
                out.push_str("  ");
                out.push_str(label);
                for _ in label.len()..width {
                    out.push(' ');
                }
                out.push_str("  ");
            } else {
                for _ in 0..width.saturating_add(4) {
                    out.push(' ');
                }
            }
            out.push_str(line);
            out.push('\n');
        }
    }
}

/// The column a next-line description starts at.
pub const WIDE_INDENT: usize = 10;

/// Each label on its own line, its description on the lines below it.
pub fn push_wide_entries(out: &mut String, entries: &[(String, String)], spaced: bool) {
    let room = TERM_WIDTH.saturating_sub(WIDE_INDENT).max(1);
    for (index, (label, desc)) in entries.iter().enumerate() {
        if spaced && index > 0 {
            out.push('\n');
        }
        out.push_str("  ");
        out.push_str(label);
        out.push('\n');
        if desc.is_empty() {
            for _ in 0..WIDE_INDENT {
                out.push(' ');
            }
            out.push('\n');
            continue;
        }
        for line in wrap(desc, room) {
            if !line.is_empty() {
                for _ in 0..WIDE_INDENT {
                    out.push(' ');
                }
                out.push_str(&line);
            }
            out.push('\n');
        }
    }
}

/// The column clap wraps option descriptions at.
pub const TERM_WIDTH: usize = 100;

/// Each authored line wrapped to `room` columns, never fewer than one line out.
pub fn wrap(desc: &str, room: usize) -> Vec<String> {
    let mut out = Vec::new();
    for paragraph in desc.lines() {
        let indent: String = paragraph
            .chars()
            .take_while(|character| *character == ' ')
            .collect();
        let mut line = indent.clone();
        let mut gap = String::new();
        for token in paragraph.trim_start_matches(' ').split(' ') {
            if token.is_empty() {
                gap.push(' ');
                continue;
            }
            let separator = if line == indent {
                String::new()
            } else {
                format!("{gap} ")
            };
            let filled = line
                .len()
                .saturating_add(separator.len())
                .saturating_add(token.len());
            if line != indent && filled > room {
                out.push(std::mem::replace(&mut line, indent.clone()));
            } else {
                line.push_str(&separator);
            }
            line.push_str(token);
            gap.clear();
        }
        out.push(line);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}
