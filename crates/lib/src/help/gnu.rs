//! The GNU coreutils dialect, a `Usage:` line and tab-separated options

use crate::def::FlagKind;
use crate::def::{Def, Flag};

use super::has_tag;

/// GNU coreutils-style help: `Usage:` line, about, then `-c, --long` options
pub fn render(def: &Def) -> String {
    let mut out = String::with_capacity(256);
    let short_doc = def.short_doc().unwrap_or("");
    let usage = if short_doc.is_empty() {
        def.usage()
    } else {
        short_doc.to_owned()
    };
    out.push_str("Usage: ");
    out.push_str(&usage);
    out.push('\n');
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
    // GNU hand-tunes each block's columns, so a command may author its whole tail
    if has_tag(def, "gnu_literal_tail") {
        out.push('\n');
        for line in def.extra() {
            out.push_str(line);
            out.push('\n');
        }
        return out;
    }
    push_options(def, &mut out);
    for line in def.exit_status() {
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Append the GNU options block; `--help`/`--version` are always listed.
pub fn push_options(def: &Def, out: &mut String) {
    out.push('\n');
    for f in def.flags().iter().filter(|f| {
        !matches!(f.flag_kind(), FlagKind::Noop) && !f.description().is_empty() && !f.is_hidden()
    }) {
        out.push_str(&flag_line(f));
    }
    if def.flags().iter().any(|flag| flag.id() == 'h') {
        out.push_str("      --help\tdisplay this help and exit\n");
    } else {
        out.push_str("  -h, --help\tdisplay this help and exit\n");
    }
    if def.flags().iter().any(|flag| flag.id() == 'V') || def.policy().no_implicit_version {
        out.push_str("      --version\toutput version information and exit\n");
    } else {
        out.push_str("  -V, --version\toutput version information and exit\n");
    }
}

/// One GNU option line: `  -c, --long[=VALUE]\tDESC`, or `      --long` when long-only.
pub fn flag_line(f: &Flag) -> String {
    let value_name = f.value_name().unwrap_or("");
    let value = if value_name.is_empty() {
        String::new()
    } else {
        format!("={value_name}")
    };
    let long = f.long_name().unwrap_or("");
    let head = if f.short().is_none() {
        format!("      --{long}{value}")
    } else if long.is_empty() {
        format!("  -{}", f.id())
    } else {
        format!("  -{}, --{long}{value}", f.id())
    };
    let marker = if f.is_implemented() {
        ""
    } else {
        " (external only)"
    };
    format!("{head}\t{}{marker}\n", f.description())
}
