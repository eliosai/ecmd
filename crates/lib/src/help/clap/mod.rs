//! The clap dialect, `Usage:`, `Arguments:` and a column-aligned `Options:`

mod wrap;

use crate::meta::{CommandDef, PositionalDef, Storage};
use crate::parse::{FlagDef, FlagKind};
use crate::style::HelpStyle;

use super::{is_synthetic, spaced_help};

/// Clap-style help: about, `Usage:`, `Arguments:`, then aligned `Options:`, never rewrapped
pub fn render<S: Storage>(def: &CommandDef<S>) -> String {
    let mut out = String::with_capacity(512);
    for line in def.about.as_ref().lines() {
        out.push_str(line);
        out.push('\n');
    }
    if !def.description.as_ref().is_empty() {
        out.push('\n');
    }
    for line in def.description.as_ref() {
        out.push_str(line.as_ref());
        out.push('\n');
    }
    push_usage(def, &mut out);
    push_arguments(def, &mut out);
    push_options(def, &mut out);
    if !def.extra.as_ref().is_empty() {
        out.push('\n');
        for line in def.extra.as_ref() {
            out.push_str(line.as_ref());
            out.push('\n');
        }
    }
    out
}

/// `Usage:` with any continuation lines indented under the first.
pub fn push_usage<S: Storage>(def: &CommandDef<S>, out: &mut String) {
    let short_doc = def.short_doc.as_ref();
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
pub fn push_arguments<S: Storage>(def: &CommandDef<S>, out: &mut String) {
    let entries = argument_entries(def);
    if entries.is_empty() {
        return;
    }
    out.push_str("\nArguments:\n");
    if def.help_style == HelpStyle::ClapWide && spaced_help(def) {
        wrap::push_wide_entries(out, &entries, false);
    } else {
        wrap::push_entries(out, &entries);
    }
}

/// One label and description per visible positional, rest slot last.
pub fn argument_entries<S: Storage>(def: &CommandDef<S>) -> Vec<(String, String)> {
    let mut entries: Vec<(String, String)> = def
        .positionals
        .as_ref()
        .iter()
        .filter(|positional| !positional.hidden)
        .map(|positional| {
            let label = positional.label.as_ref();
            let label = if label.is_empty() {
                positional.name.as_ref()
            } else {
                label
            };
            let tail = if positional.spread { "..." } else { "" };
            let label = if positional.required {
                format!("<{label}>{tail}")
            } else {
                format!("[{label}]{tail}")
            };
            (label, argument_desc(positional))
        })
        .collect();
    if def.has_rest && !def.rest_hidden {
        let label = def.rest_label.as_ref();
        let label = if label.is_empty() { "args" } else { label };
        let label = if def.rest_required {
            format!("<{label}>...")
        } else {
            format!("[{label}]...")
        };
        entries.push((label, rest_desc(def)));
    }
    insert_arg_rows(def, &mut entries);
    entries
}

/// Help-only positional rows from `arg_row` tags, each `INDEX\tLABEL\tDESC`.
pub fn insert_arg_rows<S: Storage>(def: &CommandDef<S>, entries: &mut Vec<(String, String)>) {
    for (name, value) in def.tags() {
        if name.as_ref() != "arg_row" {
            continue;
        }
        let mut parts = value.as_ref().splitn(3, '\t');
        let (Some(at), Some(label), Some(desc)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        let at = at.parse().unwrap_or(entries.len()).min(entries.len());
        entries.insert(at, (label.to_owned(), desc.to_owned()));
    }
}

/// The rest slot's description, with clap's trailing default note.
pub fn rest_desc<S: Storage>(def: &CommandDef<S>) -> String {
    let desc = def.rest_desc.as_ref();
    let default = def.rest_default.as_ref();
    match (desc.is_empty(), default.is_empty()) {
        (_, true) => desc.to_owned(),
        (true, false) => format!("[default: {default}]"),
        (false, false) => format!("{desc} [default: {default}]"),
    }
}

/// `Options:` aligned to the widest label, `--help` and `--version` last.
pub fn push_options<S: Storage>(def: &CommandDef<S>, out: &mut String) {
    let mut entries: Vec<(String, String)> = def
        .flags()
        .iter()
        .filter(|flag| {
            !matches!(flag.kind, FlagKind::Noop)
                && !flag.hidden
                && (def.help_style == HelpStyle::ClapWide
                    || !flag.desc.as_ref().is_empty()
                    || !flag.help_values.as_ref().is_empty())
        })
        .map(|flag| (flag_label(flag), flag_desc(flag)))
        .collect();
    let owns = |ch: char| def.flags().iter().any(|flag| flag.ch == ch);
    let wording = |key: &str, fallback: &str| {
        def.tags()
            .iter()
            .find_map(|(name, value)| (name.as_ref() == key).then(|| value.as_ref()))
            .unwrap_or(fallback)
            .to_owned()
    };
    entries.push((
        if owns('h') {
            "    --help".to_owned()
        } else {
            "-h, --help".to_owned()
        },
        wording("help_desc", "Print help"),
    ));
    entries.push((
        if owns('V') || def.no_implicit_version {
            "    --version".to_owned()
        } else {
            "-V, --version".to_owned()
        },
        wording("version_desc", "Print version"),
    ));
    if def
        .tags()
        .iter()
        .any(|(name, _)| name.as_ref() == "help_first")
    {
        let help = entries.remove(entries.len().saturating_sub(2));
        entries.insert(0, help);
        if def
            .tags()
            .iter()
            .any(|(name, _)| name.as_ref() == "version_first")
        {
            let version = entries.pop().unwrap_or_default();
            entries.insert(1, version);
        }
    } else if let Some(long) = def
        .tags()
        .iter()
        .find_map(|(name, value)| (name.as_ref() == "help_before").then(|| value.as_ref()))
    {
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
    if def.help_style == HelpStyle::ClapWide {
        wrap::push_wide_entries(out, &entries, spaced_help(def));
    } else {
        wrap::push_entries(out, &entries);
    }
}

/// `-c, --long <VAL>`, with the short column blank for long-only flags.
pub fn flag_label<S: Storage>(flag: &FlagDef<S>) -> String {
    let override_label = flag.help_label.as_ref();
    if !override_label.is_empty() {
        return override_label.to_owned();
    }
    let mut label = String::with_capacity(24);
    if is_synthetic(flag.ch) {
        label.push_str("    ");
    } else {
        label.push('-');
        label.push(flag.ch);
        if flag.long.as_ref().is_empty() {
            let value_name = flag.value_name.as_ref();
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
    label.push_str(flag.long.as_ref());
    let value_name = flag.value_name.as_ref();
    if !value_name.is_empty() {
        label.push_str(" <");
        label.push_str(value_name);
        label.push('>');
    }
    label
}

/// A flag's description with clap's trailing alias, default, and value notes.
pub fn flag_desc<S: Storage>(flag: &FlagDef<S>) -> String {
    let mut desc = flag.desc.as_ref().to_owned();
    let empty = desc.is_empty();
    let default = flag.default_value.as_ref();
    if !default.is_empty() {
        desc.push_str(" [default: ");
        desc.push_str(default);
        desc.push(']');
    }
    // help lists only what the author asked it to; the accepted set is a parsing fact
    let values = flag.help_values.as_ref();
    if !values.is_empty() {
        let joined = values
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join(", ");
        desc.push_str(" [possible values: ");
        desc.push_str(&joined);
        desc.push(']');
    }
    let aliases = flag.visible_aliases.as_ref();
    if !aliases.is_empty() {
        let joined = aliases
            .iter()
            .map(|alias| {
                let alias = alias.as_ref();
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
        desc = desc.trim_start().to_owned();
    }
    desc
}

/// A positional's description, with clap's trailing default note when it has one.
pub fn argument_desc<S: Storage>(positional: &PositionalDef<S>) -> String {
    let desc = positional.desc.as_ref();
    let default = positional.default_value.as_ref();
    match (desc.is_empty(), default.is_empty()) {
        (_, true) => desc.to_owned(),
        (true, false) => format!("[default: {default}]"),
        (false, false) => format!("{desc} [default: {default}]"),
    }
}
