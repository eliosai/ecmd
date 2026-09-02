//! Help and usage text in every dialect a command may render

use crate::def::FlagKind;
use crate::def::{Def, Flag};
use crate::style::HelpStyle;

mod bash;
mod clap;
mod gnu;
mod util_linux;

const INDENT: &str = "    ";

impl Def {
    /// Auto-generate a usage string like `cd [-LP] [dir]`.
    #[must_use]
    pub fn usage(&self) -> String {
        let mut parts = vec![self.name().to_owned()];
        parts.extend(flag_usage(self));
        for positional in self.positionals() {
            parts.push(if positional.is_required() {
                positional.name().to_owned()
            } else {
                format!("[{}]", positional.name())
            });
        }
        if self.rest().is_some() {
            parts.push("[args...]".to_owned());
        }
        parts.join(" ")
    }

    /// Help text in the command's style: bash-builtin for POSIX, GNU for Gnu.
    #[must_use]
    pub fn help(&self) -> String {
        // a bespoke page no dialect describes is authored whole and emitted untouched
        if has_tag(self, "verbatim_help") {
            return verbatim(self);
        }
        match self.help_style() {
            HelpStyle::Gnu => gnu::render(self),
            HelpStyle::Clap | HelpStyle::ClapWide => clap::render(self),
            HelpStyle::UtilLinux => util_linux::render(self),
            HelpStyle::Bash => bash::render(self),
        }
    }
}

/// Flag fragments for the usage line; long-only flags show as `[--long]`.
pub fn flag_usage(def: &Def) -> Vec<String> {
    let mut parts = Vec::new();
    let shorts: String = def
        .flags()
        .iter()
        .filter(|f| {
            matches!(
                f.flag_kind(),
                FlagKind::Bool | FlagKind::Polar | FlagKind::Noop
            ) && f.short().is_some()
                && !f.is_hidden()
        })
        .map(Flag::id)
        .collect();
    if !shorts.is_empty() {
        parts.push(format!("[-{shorts}]"));
    }
    for f in def.flags().iter().filter(|f| !f.is_hidden()) {
        let value_name = f.value_name().unwrap_or("");
        let vn = if value_name.is_empty() {
            "ARG"
        } else {
            value_name
        };
        if matches!(f.flag_kind(), FlagKind::Value | FlagKind::PolarValue) && f.short().is_some() {
            parts.push(format!("[-{} {vn}]", f.id()));
        } else if !f.long_name().unwrap_or("").is_empty() && f.short().is_none() {
            let val = if value_name.is_empty() {
                String::new()
            } else {
                format!("={value_name}")
            };
            parts.push(format!("[--{}{val}]", f.long_name().unwrap_or("")));
        }
    }
    parts
}

/// The tag's value, or `fallback` when the command does not carry it
pub fn tag_or<'t>(def: &'t Def, key: &str, fallback: &'t str) -> &'t str {
    def.tag(key).unwrap_or(fallback)
}

/// Whether the command carries `key` at all.
pub fn has_tag(def: &Def, key: &str) -> bool {
    def.tag(key).is_some()
}

/// The column a tagged width names, or `fallback` when it names none.
pub fn tag_width(def: &Def, key: &str, fallback: usize) -> usize {
    tag_or(def, key, "").parse().unwrap_or(fallback)
}

/// Whether help separates its entries with blank lines.
pub fn spaced_help(def: &Def) -> bool {
    has_tag(def, "help_spaced")
}

/// The authored `# Options` block, verbatim, as the entire help page.
pub fn verbatim(def: &Def) -> String {
    let mut out = String::with_capacity(512);
    for line in def.extra() {
        out.push_str(line);
        out.push('\n');
    }
    out
}

pub fn push_line(out: &mut String, indent: &str, text: &str) {
    out.push_str(indent);
    out.push_str(text);
    out.push('\n');
}

pub fn push_empty(out: &mut String) {
    out.push_str(INDENT);
    out.push('\n');
}

pub fn push_body(out: &mut String, line: &str) {
    if line.is_empty() {
        push_empty(out);
    } else {
        push_line(out, INDENT, line);
    }
}
