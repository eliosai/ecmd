//! The util-linux dialect, a leading blank, `Usage:` alone and a `(1)` trailer

use crate::meta::{CommandDef, Storage};
use crate::parse::{FlagDef, FlagKind};

use super::{has_tag, spaced_help, tag_or, tag_width};

pub fn render<S: Storage>(def: &CommandDef<S>) -> String {
    let mut out = String::with_capacity(512);
    out.push('\n');
    out.push_str("Usage:\n");
    let short_doc = def.short_doc.as_ref();
    let usage = if short_doc.is_empty() {
        def.usage()
    } else {
        short_doc.to_owned()
    };
    for line in usage.lines() {
        out.push(' ');
        out.push_str(line);
        out.push('\n');
    }
    if !def.description.as_ref().is_empty() {
        out.push('\n');
        for line in def.description.as_ref() {
            out.push_str(line.as_ref());
            out.push('\n');
        }
    }

    let entries = entries(def);
    let natural = entries
        .iter()
        .map(|(label, _)| label.chars().count().saturating_add(3))
        .max()
        .unwrap_or(16);
    let width = tag_width(def, "help_width", natural);
    out.push_str("\nOptions:\n");
    for (label, desc) in &entries {
        // a row with neither label nor description is a separator inside the block
        if label.is_empty() && desc.is_empty() {
            out.push('\n');
        } else {
            push_entry(&mut out, label, desc, width);
        }
    }

    if spaced_help(def) {
        out.push('\n');
    }
    let pair = tag_width(def, "help_pair_width", width);
    let owns = |ch: char| def.flags().iter().any(|flag| flag.ch == ch);
    let help_label = if owns('h') {
        "    --help"
    } else {
        "-h, --help"
    };
    push_entry(
        &mut out,
        help_label,
        tag_or(def, "help_desc", "display this help and exit"),
        pair,
    );
    if !owns('V') && !def.no_implicit_version {
        push_entry(
            &mut out,
            "-V, --version",
            tag_or(def, "version_desc", "output version information and exit"),
            pair,
        );
    }

    for block in def.extra.as_ref() {
        out.push('\n');
        for line in block.as_ref().lines() {
            out.push_str(line);
            out.push('\n');
        }
    }

    if !has_tag(def, "no_trailer") {
        let page = tag_or(def, "man_page", def.name());
        out.push_str("\nFor more details see ");
        out.push_str(page);
        out.push_str("(1).\n");
    }
    out
}

/// Every documented flag as util-linux labels and describes it.
pub fn entries<S: Storage>(def: &CommandDef<S>) -> Vec<(String, String)> {
    let mut entries: Vec<(String, String)> = def
        .flags()
        .iter()
        .filter(|flag| !flag.hidden && !flag.desc.as_ref().is_empty())
        .filter(|flag| !matches!(flag.long.as_ref(), "help" | "version"))
        .map(|flag| (label(flag), flag.desc.as_ref().to_owned()))
        .collect();
    for (name, value) in def.tags() {
        if name.as_ref() != "help_row" {
            continue;
        }
        let mut parts = value.as_ref().splitn(3, '\t');
        let (Some(at), Some(label), Some(desc)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        let at = at.parse().unwrap_or(entries.len()).min(entries.len());
        entries.insert(at, (label.to_owned(), desc.to_owned()));
    }
    entries
}

/// One option row, its continuation lines hanging two past the description column.
pub fn push_entry(out: &mut String, label: &str, desc: &str, width: usize) {
    let mut lines = desc.lines();
    let head = lines.next().unwrap_or("");
    let shown = format!(" {label}");
    let pad = width.saturating_sub(shown.chars().count()).max(1);
    out.push_str(&shown);
    if !head.is_empty() {
        out.push_str(&" ".repeat(pad));
        out.push_str(head);
    }
    out.push('\n');
    for line in lines {
        // an authored indent places the line itself; otherwise it hangs two past the column
        if !line.starts_with(' ') {
            out.push_str(&" ".repeat(width.saturating_add(2)));
        }
        out.push_str(line);
        out.push('\n');
    }
}

/// util-linux writes a long-only flag under the long column and brackets an optional value.
pub fn label<S: Storage>(flag: &FlagDef<S>) -> String {
    let authored = flag.help_label.as_ref();
    if !authored.is_empty() {
        return authored.to_owned();
    }
    let long = flag.long.as_ref();
    let value = flag.value_name.as_ref();
    let mut label = if flag.ch.is_ascii_graphic() {
        if long.is_empty() {
            format!("-{}", flag.ch)
        } else {
            format!("-{}, --{long}", flag.ch)
        }
    } else {
        format!("    --{long}")
    };
    if !value.is_empty() {
        if matches!(flag.kind, FlagKind::Value | FlagKind::PolarValue) {
            label.push_str(" <");
            label.push_str(value);
            label.push('>');
        } else {
            label.push_str("[=<");
            label.push_str(value);
            label.push_str(">]");
        }
    }
    label
}
