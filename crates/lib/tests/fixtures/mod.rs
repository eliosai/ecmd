//! Flag definitions the scan tests share

#![expect(dead_code, reason = "each test binary uses its own subset")]

use ecmd::parse::{FlagDef, FlagKind};

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

pub const fn no_implicit_version_marker() -> FlagDef {
    FlagDef {
        ch: '\0',
        kind: FlagKind::Noop,
        long: "\0no-implicit-version",
        aliases: &[],
        clears: &[],
        desc: "",
        value_name: "",
        hidden: true,
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
