//! Help and usage text in every dialect a command may render

use crate::meta::{CommandDef, Storage};
use crate::parse::FlagKind;
use crate::style::HelpStyle;

mod bash;
mod clap;
mod gnu;
mod util_linux;

const INDENT: &str = "    ";

impl<S: Storage> CommandDef<S> {
    /// Auto-generate a usage string like `cd [-LP] [dir]`.
    #[must_use]
    pub fn usage(&self) -> String {
        let mut parts = vec![self.name().to_owned()];
        parts.extend(flag_usage(self));
        for p in self.positionals.as_ref() {
            parts.push(if p.required {
                p.name.as_ref().to_owned()
            } else {
                format!("[{}]", p.name.as_ref())
            });
        }
        if self.has_rest {
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
        match self.help_style {
            HelpStyle::Gnu => gnu::render(self),
            HelpStyle::Clap | HelpStyle::ClapWide => clap::render(self),
            HelpStyle::UtilLinux => util_linux::render(self),
            HelpStyle::Bash => bash::render(self),
        }
    }
}

/// Flag fragments for the usage line; long-only flags show as `[--long]`.
pub fn flag_usage<S: Storage>(def: &CommandDef<S>) -> Vec<String> {
    let mut parts = Vec::new();
    let shorts: String = def
        .flags()
        .iter()
        .filter(|f| {
            matches!(f.kind, FlagKind::Bool | FlagKind::Polar | FlagKind::Noop)
                && !is_synthetic(f.ch)
                && !f.hidden
        })
        .map(|f| f.ch)
        .collect();
    if !shorts.is_empty() {
        parts.push(format!("[-{shorts}]"));
    }
    for f in def.flags().iter().filter(|f| !f.hidden) {
        let value_name = f.value_name.as_ref();
        let vn = if value_name.is_empty() {
            "ARG"
        } else {
            value_name
        };
        if matches!(f.kind, FlagKind::Value | FlagKind::PolarValue) && !is_synthetic(f.ch) {
            parts.push(format!("[-{} {vn}]", f.ch));
        } else if !f.long.as_ref().is_empty() && is_synthetic(f.ch) {
            let val = if value_name.is_empty() {
                String::new()
            } else {
                format!("={value_name}")
            };
            parts.push(format!("[--{}{val}]", f.long.as_ref()));
        }
    }
    parts
}

/// The tag's value, or `fallback` when the command does not carry it
pub fn tag_or<'t, S: Storage>(def: &'t CommandDef<S>, key: &str, fallback: &'t str) -> &'t str {
    def.tags()
        .iter()
        .find_map(|(name, value)| (name.as_ref() == key).then(|| value.as_ref()))
        .unwrap_or(fallback)
}

/// Whether the command carries `key` at all.
pub fn has_tag<S: Storage>(def: &CommandDef<S>, key: &str) -> bool {
    def.tags().iter().any(|(name, _)| name.as_ref() == key)
}

/// The column a tagged width names, or `fallback` when it names none.
pub fn tag_width<S: Storage>(def: &CommandDef<S>, key: &str, fallback: usize) -> usize {
    tag_or(def, key, "").parse().unwrap_or(fallback)
}

/// Whether help separates its entries with blank lines.
pub fn spaced_help<S: Storage>(def: &CommandDef<S>) -> bool {
    def.tags()
        .iter()
        .any(|(name, _)| name.as_ref() == "help_spaced")
}

/// The authored `# Options` block, verbatim, as the entire help page.
pub fn verbatim<S: Storage>(def: &CommandDef<S>) -> String {
    let mut out = String::with_capacity(512);
    for line in def.extra.as_ref() {
        out.push_str(line.as_ref());
        out.push('\n');
    }
    out
}

/// Long-only flags carry a synthetic codepoint at or above U+E000, past every byte-derived short
pub const fn is_synthetic(ch: char) -> bool {
    ch >= '\u{E000}'
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
