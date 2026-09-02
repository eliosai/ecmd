//! The GNU coreutils dialect, a `Usage:` line and tab-separated options

use crate::meta::{CommandDef, Storage};
use crate::parse::{FlagDef, FlagKind};

use super::{has_tag, is_synthetic};

/// GNU coreutils-style help: `Usage:` line, about, then `-c, --long` options
pub fn render<S: Storage>(def: &CommandDef<S>) -> String {
    let mut out = String::with_capacity(256);
    let short_doc = def.short_doc.as_ref();
    let usage = if short_doc.is_empty() {
        def.usage()
    } else {
        short_doc.to_owned()
    };
    out.push_str("Usage: ");
    out.push_str(&usage);
    out.push('\n');
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
    // GNU hand-tunes each block's columns, so a command may author its whole tail
    if has_tag(def, "gnu_literal_tail") {
        out.push('\n');
        for line in def.extra.as_ref() {
            out.push_str(line.as_ref());
            out.push('\n');
        }
        return out;
    }
    push_options(def, &mut out);
    for line in def.exit_status.as_ref() {
        out.push_str(line.as_ref());
        out.push('\n');
    }
    out
}

/// Append the GNU options block; `--help`/`--version` are always listed.
pub fn push_options<S: Storage>(def: &CommandDef<S>, out: &mut String) {
    out.push('\n');
    for f in def
        .flags()
        .iter()
        .filter(|f| !matches!(f.kind, FlagKind::Noop) && !f.desc.as_ref().is_empty() && !f.hidden)
    {
        out.push_str(&flag_line(f));
    }
    if def.flags().iter().any(|flag| flag.ch == 'h') {
        out.push_str("      --help\tdisplay this help and exit\n");
    } else {
        out.push_str("  -h, --help\tdisplay this help and exit\n");
    }
    if def.flags().iter().any(|flag| flag.ch == 'V') || def.no_implicit_version {
        out.push_str("      --version\toutput version information and exit\n");
    } else {
        out.push_str("  -V, --version\toutput version information and exit\n");
    }
}

/// One GNU option line: `  -c, --long[=VALUE]\tDESC`, or `      --long` when long-only.
pub fn flag_line<S: Storage>(f: &FlagDef<S>) -> String
where
    S::Text: AsRef<str>,
{
    let value_name = f.value_name.as_ref();
    let value = if value_name.is_empty() {
        String::new()
    } else {
        format!("={value_name}")
    };
    let long = f.long.as_ref();
    let head = if is_synthetic(f.ch) {
        format!("      --{long}{value}")
    } else if long.is_empty() {
        format!("  -{}", f.ch)
    } else {
        format!("  -{}, --{long}{value}", f.ch)
    };
    let marker = if f.implemented {
        ""
    } else {
        " (external only)"
    };
    format!("{head}\t{}{marker}\n", f.desc.as_ref())
}
