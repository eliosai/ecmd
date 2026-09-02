//! The static shape a derived definition is written in

use std::borrow::Cow;

use super::{Def, Flag, FlagKind, Policy, Positional, Tag};
use crate::style::{HelpStyle, Style};

/// Every field of a definition as static borrows
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
#[expect(clippy::exhaustive_structs, reason = "derive plumbing")]
pub struct RawDef {
    pub name: &'static str,
    pub about: &'static str,
    pub short_doc: &'static str,
    pub style: Style,
    pub help_style: HelpStyle,
    pub lenient: bool,
    pub permute: bool,
    pub flags: &'static [Flag],
    pub positionals: &'static [Positional],
    pub rest: Option<&'static Positional>,
    pub policy: &'static Policy,
    pub tags: &'static [Tag],
    pub description: &'static [Cow<'static, str>],
    pub extra: &'static [Cow<'static, str>],
    pub exit_status: &'static [Cow<'static, str>],
}

/// Every field of a flag as static borrows
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
#[expect(
    clippy::exhaustive_structs,
    clippy::struct_excessive_bools,
    reason = "derive plumbing"
)]
pub struct RawFlag {
    pub id: char,
    pub long: &'static str,
    pub aliases: &'static [Cow<'static, str>],
    pub kind: FlagKind,
    pub desc: &'static str,
    pub value_name: &'static str,
    pub hidden: bool,
    pub implemented: bool,
    pub repeatable: bool,
    pub allow_hyphen_values: bool,
    pub possible_values: &'static [Cow<'static, str>],
    pub help_values: &'static [Cow<'static, str>],
    pub default_value: &'static str,
    pub help_label: &'static str,
    pub visible_aliases: &'static [Cow<'static, str>],
}

/// Every field of a positional as static borrows
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
#[expect(clippy::exhaustive_structs, reason = "derive plumbing")]
pub struct RawPositional {
    pub name: &'static str,
    pub required: bool,
    pub desc: &'static str,
    pub label: &'static str,
    pub default_value: &'static str,
    pub hidden: bool,
    pub spread: bool,
}

impl Def {
    #[doc(hidden)]
    #[must_use]
    pub const fn from_raw(raw: RawDef) -> Self {
        Self {
            name: Cow::Borrowed(raw.name),
            about: Cow::Borrowed(raw.about),
            short_doc: Cow::Borrowed(raw.short_doc),
            style: raw.style,
            help_style: raw.help_style,
            lenient: raw.lenient,
            permute: raw.permute,
            flags: Cow::Borrowed(raw.flags),
            positionals: Cow::Borrowed(raw.positionals),
            rest: match raw.rest {
                Some(rest) => Some(Cow::Borrowed(rest)),
                None => None,
            },
            policy: Cow::Borrowed(raw.policy),
            tags: Cow::Borrowed(raw.tags),
            description: Cow::Borrowed(raw.description),
            extra: Cow::Borrowed(raw.extra),
            exit_status: Cow::Borrowed(raw.exit_status),
        }
    }
}

impl RawFlag {
    /// A flag with nothing set, so a literal names only what differs
    pub const EMPTY: Self = Self {
        id: '\0',
        long: "",
        aliases: &[],
        kind: FlagKind::Bool,
        desc: "",
        value_name: "",
        hidden: false,
        implemented: true,
        repeatable: true,
        allow_hyphen_values: true,
        possible_values: &[],
        help_values: &[],
        default_value: "",
        help_label: "",
        visible_aliases: &[],
    };
}

impl RawPositional {
    /// A slot with nothing set, so a literal names only what differs
    pub const EMPTY: Self = Self {
        name: "",
        required: false,
        desc: "",
        label: "",
        default_value: "",
        hidden: false,
        spread: false,
    };
}
