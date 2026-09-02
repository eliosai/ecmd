//! Definitions the scan and help tests share

#![expect(dead_code, reason = "each test binary uses its own subset")]

use ecmd::{Def, Flag, FlagKind, Style};

pub const fn bool_flag(ch: char) -> Flag {
    Flag::new(ch).once()
}

pub const fn value_flag(ch: char) -> Flag {
    Flag::new(ch).kind(FlagKind::Value).once()
}

pub const fn polar_flag(ch: char) -> Flag {
    Flag::new(ch).kind(FlagKind::Polar).once()
}

pub const fn noop_flag(ch: char) -> Flag {
    Flag::new(ch).kind(FlagKind::Noop).once()
}

pub fn long_bool(ch: char, long: &'static str) -> Flag {
    Flag::new(ch).long(long).once()
}

pub fn long_value(ch: char, long: &'static str) -> Flag {
    Flag::new(ch).long(long).kind(FlagKind::Value).once()
}

pub fn aliased_bool(ch: char, long: &'static str, aliases: &'static [&'static str]) -> Flag {
    aliases
        .iter()
        .fold(long_bool(ch, long), |flag, alias| flag.alias(*alias))
}

/// A definition over these flags with the given leniency, style and permutation
pub fn scanner(flags: &[Flag], lenient: bool, style: Style, permute: bool) -> Def {
    let builder = Def::builder("test").style(style).permute(permute);
    let builder = if lenient { builder.lenient() } else { builder };
    builder.flags(flags.iter().cloned()).build()
}

/// A bash builtin definition in the shape the `help` tests describe
pub fn make_def(
    name: &'static str,
    about: &'static str,
    short_doc: &'static str,
    flags: &[Flag],
    description: &[&'static str],
    extra: &[&'static str],
    exit_status: &[&'static str],
) -> Def {
    Def::builder(name)
        .about(about)
        .short_doc(short_doc)
        .permute(false)
        .flags(flags.iter().cloned())
        .description(description.iter().copied())
        .extra(extra.iter().copied())
        .exit_status(exit_status.iter().copied())
        .build()
}

pub fn alias_flags() -> Vec<Flag> {
    vec![
        Flag::new('p')
            .desc("print all defined aliases in a reusable format")
            .once(),
    ]
}

pub fn gnu_flags() -> Vec<Flag> {
    vec![
        Flag::new('a')
            .long("multiple")
            .desc("support multiple arguments and treat each as a NAME")
            .once(),
        Flag::new('s')
            .long("suffix")
            .value("SUFFIX")
            .desc("remove a trailing SUFFIX; implies -a")
            .once(),
    ]
}

/// A GNU definition over these flags
pub fn gnu_definition(flags: &[Flag]) -> Def {
    Def::builder("sample")
        .about("sample")
        .short_doc("sample [OPTION]...")
        .style(Style::Gnu)
        .flags(flags.iter().cloned())
        .build()
}
