//! Flag definitions the scan tests share

#![expect(dead_code, reason = "each test binary uses its own subset")]

use ecmd::meta::CommandDef;
use ecmd::parse::{FlagDef, FlagKind, OnUnknown};
use ecmd::style::{HelpStyle, Style};

pub const fn bool_flag(ch: char) -> FlagDef {
    FlagDef {
        ch,
        kind: FlagKind::Bool,
        long: "",
        aliases: &[],
        clears: &[],
        desc: "",
        value_name: "",
        hidden: false,
        implemented: true,
        repeatable: false,
        allow_hyphen_values: true,
        possible_values: &[],
        help_values: &[],
        default_value: "",
        help_label: "",
        visible_aliases: &[],
    }
}

pub const fn value_flag(ch: char) -> FlagDef {
    FlagDef {
        ch,
        kind: FlagKind::Value,
        long: "",
        aliases: &[],
        clears: &[],
        desc: "",
        value_name: "",
        hidden: false,
        implemented: true,
        repeatable: false,
        allow_hyphen_values: true,
        possible_values: &[],
        help_values: &[],
        default_value: "",
        help_label: "",
        visible_aliases: &[],
    }
}

pub const fn polar_flag(ch: char) -> FlagDef {
    FlagDef {
        ch,
        kind: FlagKind::Polar,
        long: "",
        aliases: &[],
        clears: &[],
        desc: "",
        value_name: "",
        hidden: false,
        implemented: true,
        repeatable: false,
        allow_hyphen_values: true,
        possible_values: &[],
        help_values: &[],
        default_value: "",
        help_label: "",
        visible_aliases: &[],
    }
}

pub const fn noop_flag(ch: char) -> FlagDef {
    FlagDef {
        ch,
        kind: FlagKind::Noop,
        long: "",
        aliases: &[],
        clears: &[],
        desc: "",
        value_name: "",
        hidden: false,
        implemented: true,
        repeatable: false,
        allow_hyphen_values: true,
        possible_values: &[],
        help_values: &[],
        default_value: "",
        help_label: "",
        visible_aliases: &[],
    }
}

pub const fn long_bool(ch: char, long: &'static str) -> FlagDef {
    FlagDef {
        ch,
        long,
        aliases: &[],
        kind: FlagKind::Bool,
        clears: &[],
        desc: "",
        value_name: "",
        hidden: false,
        implemented: true,
        repeatable: false,
        allow_hyphen_values: true,
        possible_values: &[],
        help_values: &[],
        default_value: "",
        help_label: "",
        visible_aliases: &[],
    }
}

pub const fn long_value(ch: char, long: &'static str) -> FlagDef {
    FlagDef {
        ch,
        long,
        aliases: &[],
        kind: FlagKind::Value,
        clears: &[],
        desc: "",
        value_name: "",
        hidden: false,
        implemented: true,
        repeatable: false,
        allow_hyphen_values: true,
        possible_values: &[],
        help_values: &[],
        default_value: "",
        help_label: "",
        visible_aliases: &[],
    }
}

pub const fn aliased_bool(
    ch: char,
    long: &'static str,
    aliases: &'static [&'static str],
) -> FlagDef {
    FlagDef {
        ch,
        long,
        aliases,
        kind: FlagKind::Bool,
        clears: &[],
        desc: "",
        value_name: "",
        hidden: false,
        implemented: true,
        repeatable: false,
        allow_hyphen_values: true,
        possible_values: &[],
        help_values: &[],
        default_value: "",
        help_label: "",
        visible_aliases: &[],
    }
}

pub const fn make_def(
    name: &'static str,
    about: &'static str,
    short_doc: &'static str,
    flags: &'static [FlagDef],
    description: &'static [&'static str],
    extra: &'static [&'static str],
    exit_status: &'static [&'static str],
) -> CommandDef {
    CommandDef {
        name,
        about,
        short_doc,
        style: Style::Posix,
        help_style: HelpStyle::from_parse_style(Style::Posix),
        on_unknown: OnUnknown::Reject,
        permute: false,
        flags,
        positionals: &[],
        has_rest: false,
        rest_label: "",
        rest_hidden: false,
        rest_desc: "",
        rest_default: "",
        rest_required: false,
        value_rules: &[],
        numeric_operands: &[],
        first_numeric_value: None,
        exact_long: false,
        no_implicit_version: false,
        equals_only: &[],
        attached_values: &[],
        separated_values: &[],
        prefixed_values: &[],
        exclusive_groups: &[],
        tags: &[],
        description,
        extra,
        exit_status,
    }
}

// ── Test against real `bash -c 'help alias'` output ────────
pub static ALIAS_FLAGS: [FlagDef; 1] = [FlagDef {
    ch: 'p',
    long: "",
    aliases: &[],
    kind: FlagKind::Bool,
    clears: &[],
    desc: "print all defined aliases in a reusable format",
    value_name: "",
    hidden: false,
    implemented: true,
    repeatable: false,
    allow_hyphen_values: true,
    possible_values: &[],
    help_values: &[],
    default_value: "",
    help_label: "",
    visible_aliases: &[],
}];

// ── GNU help formatting (Style::Gnu) ───────────────────────
pub static GNU_FLAGS: [FlagDef; 2] = [
    FlagDef {
        ch: 'a',
        long: "multiple",
        aliases: &[],
        kind: FlagKind::Bool,
        clears: &[],
        desc: "support multiple arguments and treat each as a NAME",
        value_name: "",
        hidden: false,
        implemented: true,
        repeatable: false,
        allow_hyphen_values: true,
        possible_values: &[],
        help_values: &[],
        default_value: "",
        help_label: "",
        visible_aliases: &[],
    },
    FlagDef {
        ch: 's',
        long: "suffix",
        aliases: &[],
        kind: FlagKind::Value,
        clears: &[],
        desc: "remove a trailing SUFFIX; implies -a",
        value_name: "SUFFIX",
        hidden: false,
        implemented: true,
        repeatable: false,
        allow_hyphen_values: true,
        possible_values: &[],
        help_values: &[],
        default_value: "",
        help_label: "",
        visible_aliases: &[],
    },
];

pub const fn gnu_definition(flags: &'static [FlagDef]) -> CommandDef {
    CommandDef {
        name: "sample",
        about: "sample",
        short_doc: "sample [OPTION]...",
        style: Style::Gnu,
        help_style: HelpStyle::from_parse_style(Style::Gnu),
        on_unknown: OnUnknown::Reject,
        permute: true,
        flags,
        positionals: &[],
        has_rest: false,
        rest_label: "",
        rest_hidden: false,
        rest_desc: "",
        rest_default: "",
        rest_required: false,
        value_rules: &[],
        numeric_operands: &[],
        first_numeric_value: None,
        exact_long: false,
        no_implicit_version: false,
        equals_only: &[],
        attached_values: &[],
        separated_values: &[],
        prefixed_values: &[],
        exclusive_groups: &[],
        tags: &[],
        description: &[],
        extra: &[],
        exit_status: &[],
    }
}
