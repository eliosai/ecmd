//! The clap dialect, `Usage:`, `Arguments:` and a column-aligned `Options:`

mod wrap;

use crate::def::FlagKind;
use crate::def::{Def, Flag, Positional};
use crate::style::HelpStyle;

use super::spaced_help;

/// Clap-style help: about, `Usage:`, `Arguments:`, then aligned `Options:`, never rewrapped
pub fn render(def: &Def) -> String {
    let mut out = String::with_capacity(512);
    for line in def.about().lines() {
        out.push_str(line);
        out.push('\n');
    }
    if def.description().len() > 0 {
        out.push('\n');
    }
    for line in def.description() {
        out.push_str(line);
        out.push('\n');
    }
    push_usage(def, &mut out);
    push_arguments(def, &mut out);
    push_options(def, &mut out);
    if def.extra().len() > 0 {
        out.push('\n');
        for line in def.extra() {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// `Usage:` with any continuation lines indented under the first.
pub fn push_usage(def: &Def, out: &mut String) {
    let short_doc = def.short_doc().unwrap_or("");
    let usage = if short_doc.is_empty() {
        def.usage()
    } else {
        short_doc.to_owned()
    };
    out.push('\n');
    for (index, line) in usage.lines().enumerate() {
        out.push_str(if index == 0 { "Usage: " } else { "       " });
        out.push_str(line);
        out.push('\n');
    }
}

/// `Arguments:` listing declared positionals and the rest slot.
pub fn push_arguments(def: &Def, out: &mut String) {
    let entries = argument_entries(def);
    if entries.is_empty() {
        return;
    }
    out.push_str("\nArguments:\n");
    if def.help_style() == HelpStyle::ClapWide && spaced_help(def) {
        wrap::push_wide_entries(out, &entries, false);
    } else {
        wrap::push_entries(out, &entries);
    }
}

/// One label and description per visible positional, rest slot last.
pub fn argument_entries(def: &Def) -> Vec<(String, String)> {
    let mut entries: Vec<(String, String)> = def
        .positionals()
        .iter()
        .filter(|positional| !positional.is_hidden())
        .map(|positional| {
            let label = positional.shown_as();
            let tail = if positional.is_spread() { "..." } else { "" };
            let label = if positional.is_required() {
                format!("<{label}>{tail}")
            } else {
                format!("[{label}]{tail}")
            };
            (label, argument_desc(positional))
        })
        .collect();
    if let Some(rest) = def.rest().filter(|rest| !rest.is_hidden()) {
        let label = rest.shown_as();
        let label = if rest.is_required() {
            format!("<{label}>...")
        } else {
            format!("[{label}]...")
        };
        entries.push((label, argument_desc(rest)));
    }
    insert_arg_rows(def, &mut entries);
    entries
}

/// Help-only positional rows from `arg_row` tags, each `INDEX\tLABEL\tDESC`.
pub fn insert_arg_rows(def: &Def, entries: &mut Vec<(String, String)>) {
    for (name, value) in def.tags() {
        if name != "arg_row" {
            continue;
        }
        let mut parts = value.splitn(3, '\t');
        let (Some(at), Some(label), Some(desc)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        let at = at.parse().unwrap_or(entries.len()).min(entries.len());
        entries.insert(at, (label.to_owned(), desc.to_owned()));
    }
}

/// `Options:` aligned to the widest label, `--help` and `--version` last.
pub fn push_options(def: &Def, out: &mut String) {
    let mut entries: Vec<(String, String)> = def
        .flags()
        .iter()
        .filter(|flag| {
            !matches!(flag.flag_kind(), FlagKind::Noop)
                && !flag.is_hidden()
                && (def.help_style() == HelpStyle::ClapWide
                    || !flag.description().is_empty()
                    || flag.listed_values().len() > 0)
        })
        .map(|flag| (flag_label(flag), flag_desc(flag)))
        .collect();
    let owns = |ch: char| def.flags().iter().any(|flag| flag.id() == ch);
    let wording = |key: &str, fallback: &str| def.tag(key).unwrap_or(fallback).to_owned();
    entries.push((
        if owns('h') {
            "    --help".to_owned()
        } else {
            "-h, --help".to_owned()
        },
        wording("help_desc", "Print help"),
    ));
    entries.push((
        if owns('V') || def.policy().no_implicit_version {
            "    --version".to_owned()
        } else {
            "-V, --version".to_owned()
        },
        wording("version_desc", "Print version"),
    ));
    if def.tag("help_first").is_some() {
        let help = entries.remove(entries.len().saturating_sub(2));
        entries.insert(0, help);
        if def.tag("version_first").is_some() {
            let version = entries.pop().unwrap_or_default();
            entries.insert(1, version);
        }
    } else if let Some(long) = def.tag("help_before") {
        let needle = format!("--{long}");
        if let Some(position) = entries.iter().position(|(label, _)| {
            label
                .split_whitespace()
                .any(|word| word.trim_end_matches(',') == needle)
        }) {
            let help = entries.remove(entries.len().saturating_sub(2));
            entries.insert(position, help);
        }
    }
    out.push_str("\nOptions:\n");
    if def.help_style() == HelpStyle::ClapWide {
        wrap::push_wide_entries(out, &entries, spaced_help(def));
    } else {
        wrap::push_entries(out, &entries);
    }
}

/// `-c, --long <VAL>`, with the short column blank for long-only flags.
pub fn flag_label(flag: &Flag) -> String {
    let override_label = flag.label().unwrap_or("");
    if !override_label.is_empty() {
        return override_label.to_owned();
    }
    let mut label = String::with_capacity(24);
    if flag.short().is_none() {
        label.push_str("    ");
    } else {
        label.push('-');
        label.push(flag.id());
        if flag.long_name().unwrap_or("").is_empty() {
            let value_name = flag.value_name().unwrap_or("");
            if !value_name.is_empty() {
                label.push_str(" <");
                label.push_str(value_name);
                label.push('>');
            }
            return label;
        }
        label.push_str(", ");
    }
    label.push_str("--");
    label.push_str(flag.long_name().unwrap_or(""));
    let value_name = flag.value_name().unwrap_or("");
    if !value_name.is_empty() {
        label.push_str(" <");
        label.push_str(value_name);
        label.push('>');
    }
    label
}

/// A flag's description with clap's trailing alias, default, and value notes.
pub fn flag_desc(flag: &Flag) -> String {
    let mut desc = flag.description().to_owned();
    let empty = desc.is_empty();
    let default = flag.default().unwrap_or("");
    if !default.is_empty() {
        desc.push_str(" [default: ");
        desc.push_str(default);
        desc.push(']');
    }
    // help lists only what the author asked it to; the accepted set is a parsing fact
    let values: Vec<&str> = flag.listed_values().collect();
    if !values.is_empty() {
        let joined = values.join(", ");
        desc.push_str(" [possible values: ");
        desc.push_str(&joined);
        desc.push(']');
    }
    let aliases: Vec<&str> = flag.visible_aliases().collect();
    if !aliases.is_empty() {
        let joined = aliases
            .iter()
            .map(|alias| {
                if alias.chars().count() == 1 {
                    format!("-{alias}")
                } else {
                    format!("--{alias}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let label = if aliases.len() == 1 {
            "alias"
        } else {
            "aliases"
        };
        desc.push_str(" [");
        desc.push_str(label);
        desc.push_str(": ");
        desc.push_str(&joined);
        desc.push(']');
    }
    if empty {
        let lead = desc.len().saturating_sub(desc.trim_start().len());
        desc.replace_range(..lead, "");
    }
    desc
}

/// A positional's description, with clap's trailing default note when it has one.
pub fn argument_desc(positional: &Positional) -> String {
    let desc = positional.description();
    let default = positional.default().unwrap_or("");
    match (desc.is_empty(), default.is_empty()) {
        (_, true) => desc.to_owned(),
        (true, false) => format!("[default: {default}]"),
        (false, false) => format!("{desc} [default: {default}]"),
    }
}
