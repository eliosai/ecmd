//! The bash builtin dialect, `name: usage` over an indented body

use crate::def::FlagKind;
use crate::def::{Def, Flag};

use super::{INDENT, push_body, push_empty, push_line};

pub fn render(def: &Def) -> String {
    let mut out = String::with_capacity(512);

    let short_doc = def.short_doc().unwrap_or("");
    let sd = if short_doc.is_empty() {
        def.usage()
    } else {
        short_doc.to_owned()
    };
    push_line(&mut out, "", &format!("{}: {sd}", def.name()));

    for line in def.about().lines() {
        push_line(&mut out, INDENT, line);
    }

    if def.description().len() > 0 {
        push_empty(&mut out);
        for line in def.description() {
            push_body(&mut out, line);
        }
    }

    let printable: Vec<_> = def
        .flags()
        .iter()
        .filter(|f| {
            !matches!(f.flag_kind(), FlagKind::Noop)
                && !f.description().is_empty()
                && !f.is_hidden()
        })
        .collect();
    let has_options = !printable.is_empty();
    if has_options {
        push_empty(&mut out);
        push_line(&mut out, INDENT, "Options:");
        for f in &printable {
            push_flag(&mut out, f);
        }
    }

    if def.extra().len() > 0 {
        push_empty(&mut out);
        for line in def.extra() {
            push_body(&mut out, line);
        }
    }

    if def.exit_status().len() > 0 {
        push_empty(&mut out);
        push_line(&mut out, INDENT, "Exit Status:");
        for line in def.exit_status() {
            push_body(&mut out, line);
        }
    }

    out
}

pub fn push_flag(out: &mut String, f: &Flag) {
    let value_name = f.value_name().unwrap_or("");
    let flag_part = if value_name.is_empty() {
        format!("-{}", f.id())
    } else {
        format!("-{} {value_name}", f.id())
    };

    let desc = f.description();
    let desc_lines: Vec<&str> = if desc.is_empty() {
        vec![]
    } else {
        desc.split('\n').collect()
    };
    let marker = if f.is_implemented() {
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
