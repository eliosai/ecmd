//! The bash builtin dialect, `name: usage` over an indented body

use crate::meta::{CommandDef, Storage};
use crate::parse::{FlagDef, FlagKind};

use super::{INDENT, push_body, push_empty, push_line};

pub fn render<S: Storage>(def: &CommandDef<S>) -> String {
    let mut out = String::with_capacity(512);

    let short_doc = def.short_doc.as_ref();
    let sd = if short_doc.is_empty() {
        def.usage()
    } else {
        short_doc.to_owned()
    };
    push_line(&mut out, "", &format!("{}: {sd}", def.name()));

    for line in def.about.as_ref().lines() {
        push_line(&mut out, INDENT, line);
    }

    if !def.description.as_ref().is_empty() {
        push_empty(&mut out);
        for line in def.description.as_ref() {
            push_body(&mut out, line.as_ref());
        }
    }

    let printable: Vec<_> = def
        .flags()
        .iter()
        .filter(|f| !matches!(f.kind, FlagKind::Noop) && !f.desc.as_ref().is_empty() && !f.hidden)
        .collect();
    let has_options = !printable.is_empty();
    if has_options {
        push_empty(&mut out);
        push_line(&mut out, INDENT, "Options:");
        for f in &printable {
            push_flag(&mut out, f);
        }
    }

    if !def.extra.as_ref().is_empty() {
        push_empty(&mut out);
        for line in def.extra.as_ref() {
            push_body(&mut out, line.as_ref());
        }
    }

    if !def.exit_status.as_ref().is_empty() {
        push_empty(&mut out);
        push_line(&mut out, INDENT, "Exit Status:");
        for line in def.exit_status.as_ref() {
            push_body(&mut out, line.as_ref());
        }
    }

    out
}

pub fn push_flag<S: Storage>(out: &mut String, f: &FlagDef<S>)
where
    S::Text: AsRef<str>,
{
    let value_name = f.value_name.as_ref();
    let flag_part = if value_name.is_empty() {
        format!("-{}", f.ch)
    } else {
        format!("-{} {value_name}", f.ch)
    };

    let desc = f.desc.as_ref();
    let desc_lines: Vec<&str> = if desc.is_empty() {
        vec![]
    } else {
        desc.split('\n').collect()
    };
    let marker = if f.implemented {
        ""
    } else {
        " (external only)"
    };

    let mut lines = desc_lines.iter();
    if let Some(first) = lines.next() {
        push_line(out, INDENT, &format!("  {flag_part}\t{first}{marker}"));
        for cont in lines {
            push_line(out, INDENT, &format!("\t\t{cont}"));
        }
    } else {
        push_line(out, INDENT, &format!("  {flag_part}{marker}"));
    }
}
