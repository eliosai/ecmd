//! Short flag clusters, values, polarity and pass-through

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests verify success paths"
)]

use ecmd::{Def, Error, Flag, FlagKind, Parsed, Polarity, Spelling, Style};
mod fixtures;

use pretty_assertions::assert_eq;

use fixtures::{bool_flag, noop_flag, polar_flag, scanner, value_flag};

#[test]
fn empty_args_returns_empty() {
    let def = scanner(&[], false, Style::Posix, true);
    let r = def.scan(&[]).unwrap();
    assert!(r.flags().is_empty());
    assert!(r.operands().is_empty());
}

#[test]
fn single_bool_flag() {
    let flags = [bool_flag('v')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-v"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('v')]);
    assert!(r.operands().is_empty());
}

#[test]
fn bundled_bool_flags() {
    let flags = [bool_flag('a'), bool_flag('b'), bool_flag('c')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-abc"]).unwrap();
    assert_eq!(
        r.flags(),
        [Parsed::Bool('a'), Parsed::Bool('b'), Parsed::Bool('c'),]
    );
}

#[test]
fn double_dash_terminates_flags() {
    let flags = [bool_flag('v')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["--", "-v"]).unwrap();
    assert!(r.flags().is_empty());
    assert_eq!(r.operands(), ["-v"]);
}

#[test]
fn operands_after_flags() {
    let flags = [bool_flag('v')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-v", "file.txt"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('v')]);
    assert_eq!(r.operands(), ["file.txt"]);
}

#[test]
fn valued_flag_separate() {
    let flags = [value_flag('o')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-o", "file"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('o', "file")]);
    assert!(r.operands().is_empty());
}

#[test]
fn valued_flag_separate_with_trailing() {
    let flags = [value_flag('o')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-o", "file", "rest"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('o', "file")]);
    assert_eq!(r.operands(), ["rest"]);
}

#[test]
fn valued_flag_stuck() {
    let flags = [value_flag('o')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-ofile"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('o', "file")]);
    assert!(r.operands().is_empty());
}

#[test]
fn valued_flag_stuck_strips_equals_separator() {
    let flags = [value_flag('o')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-o=file"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('o', "file")]);
    assert!(r.operands().is_empty());
}

#[test]
fn valued_flag_stuck_strips_only_one_equals() {
    let flags = [value_flag('o')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-o==file"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('o', "=file")]);
}

#[test]
fn valued_flag_stuck_bare_equals_is_empty_value() {
    let flags = [value_flag('o')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-o="]).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('o', "")]);
}

#[test]
fn valued_flag_missing_arg_errors() {
    let flags = [value_flag('o')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-o"]);
    assert!(matches!(r, Err(Error::MissingValue(_))));
}

#[test]
fn bundled_bool_then_valued() {
    let flags = [bool_flag('v'), value_flag('o')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-vo", "file"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('v'), Parsed::Value('o', "file"),]);
    assert!(r.operands().is_empty());
}

#[test]
fn bundled_bool_then_valued_stuck() {
    let flags = [bool_flag('v'), value_flag('o')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-vofile"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('v'), Parsed::Value('o', "file"),]);
    assert!(r.operands().is_empty());
}

#[test]
fn multiple_valued_flags_in_sequence() {
    let flags = [value_flag('o'), value_flag('d')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-o", "out", "-d", "dir"]).unwrap();
    assert_eq!(
        r.flags(),
        [Parsed::Value('o', "out"), Parsed::Value('d', "dir"),]
    );
    assert!(r.operands().is_empty());
}

#[test]
fn unknown_flag_errors() {
    let flags = [bool_flag('v')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-x"]);
    assert!(matches!(r, Err(Error::UnknownFlag(_))));
}

#[test]
fn gnu_implicit_short_help_and_version_are_actions() {
    for (arg, expected) in [
        ("-h", Error::HelpRequested),
        ("-V", Error::VersionRequested),
    ] {
        let args = [arg];
        let def = scanner(&[], false, Style::Gnu, true);
        let result = def.scan(&args);
        assert_eq!(result.unwrap_err(), expected);
    }
}

#[test]
fn gnu_reserved_actions_reject_inline_values() {
    for option in ["--help=value", "--version=value"] {
        let args = [option];
        let def = scanner(&[], false, Style::Gnu, true);
        let result = def.scan(&args);
        let (flag, value) = option.split_once('=').unwrap();
        assert_eq!(
            result.unwrap_err(),
            Error::UnexpectedValue {
                flag: flag.to_owned(),
                value: value.to_owned(),
            }
        );
    }
}

#[test]
fn no_implicit_version_rejects_short_v_but_keeps_help_and_long_version() {
    let def = Def::builder("")
        .style(Style::Gnu)
        .permute(false)
        .no_implicit_version()
        .build();
    assert_eq!(
        def.scan(&["-V"]).unwrap_err(),
        Error::UnknownFlag("-V".to_owned())
    );
    assert_eq!(def.scan(&["-h"]).unwrap_err(), Error::HelpRequested);
    assert_eq!(
        def.scan(&["--version"]).unwrap_err(),
        Error::VersionRequested
    );
}

#[test]
fn declared_gnu_short_help_and_version_characters_win() {
    let flags = [bool_flag('h'), bool_flag('V')];
    let def = scanner(&flags, false, Style::Gnu, true);
    let result = def.scan(&["-hV"]).unwrap();
    assert_eq!(result.flags(), [Parsed::Bool('h'), Parsed::Bool('V')]);
}

#[test]
fn posix_does_not_add_implicit_short_actions() {
    let def = scanner(&[], false, Style::Posix, true);
    let result = def.scan(&["-h"]);
    assert_eq!(result.unwrap_err(), Error::UnknownFlag("-h".to_owned()));
}

#[test]
fn passthrough_unknown_becomes_operand() {
    let flags = [bool_flag('n')];
    let def = scanner(&flags, true, Style::Posix, true);
    let r = def.scan(&["-nea"]).unwrap();
    assert!(r.flags().is_empty());
    assert_eq!(r.operands(), ["-nea"]);
}

#[test]
fn passthrough_all_known_still_parses() {
    let flags = [bool_flag('n'), bool_flag('e')];
    let def = scanner(&flags, true, Style::Posix, true);
    let r = def.scan(&["-ne", "hello"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('n'), Parsed::Bool('e')]);
    assert_eq!(r.operands(), ["hello"]);
}

#[test]
fn passthrough_first_char_unknown() {
    let flags = [bool_flag('n')];
    let def = scanner(&flags, true, Style::Posix, true);
    let r = def.scan(&["-xyz", "rest"]).unwrap();
    assert!(r.flags().is_empty());
    assert_eq!(r.operands(), ["-xyz", "rest"]);
}

#[test]
fn passthrough_value_flag_stuck_still_parses() {
    let flags = [value_flag('o')];
    let def = scanner(&flags, true, Style::Posix, true);
    let r = def.scan(&["-ofile"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('o', "file")]);
    assert!(r.operands().is_empty());
}

#[test]
fn polarity_on() {
    let flags = [polar_flag('x')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-x"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Polar('x', Polarity::On)]);
}

#[test]
fn polarity_off() {
    let flags = [polar_flag('x')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["+x"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Polar('x', Polarity::Off)]);
}

#[test]
fn plus_prefix_only_when_polarity_flags_exist() {
    let flags = [bool_flag('x')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["+x"]).unwrap();
    assert_eq!(r.operands(), ["+x"]);
}

#[test]
fn noop_flag_accepted_silently() {
    let flags = [bool_flag('r'), noop_flag('e')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-re"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('r')]);
}

#[test]
fn noop_flag_repetition_is_rejected_in_gnu_style() {
    let flags = [noop_flag('f')];
    let def = scanner(&flags, false, Style::Gnu, true);
    let error = def
        .scan(&["-f", "-f"])
        .expect_err("a scalar noop may occur once");
    assert_eq!(error, Error::RepeatedFlag("-f".to_owned()));
}

#[test]
fn bare_dash_is_operand() {
    let flags = [bool_flag('v')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-"]).unwrap();
    assert_eq!(r.operands(), ["-"]);
}

#[test]
fn multiple_flag_groups() {
    let flags = [bool_flag('a'), bool_flag('b')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-a", "-b", "file"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('a'), Parsed::Bool('b')]);
    assert_eq!(r.operands(), ["file"]);
}

#[test]
fn polar_value_on() {
    let flags: [Flag; 1] = [Flag::new('o').kind(FlagKind::PolarValue).once()];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-o", "errexit"]).unwrap();
    assert_eq!(
        r.flags(),
        [Parsed::PolarValue('o', Polarity::On, "errexit")]
    );
    assert!(r.operands().is_empty());
}

#[test]
fn polar_value_off() {
    let flags: [Flag; 1] = [Flag::new('o').kind(FlagKind::PolarValue).once()];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["+o", "verbose"]).unwrap();
    assert_eq!(
        r.flags(),
        [Parsed::PolarValue('o', Polarity::Off, "verbose")]
    );
    assert!(r.operands().is_empty());
}

#[test]
fn polar_value_stuck() {
    let flags: [Flag; 1] = [Flag::new('o').kind(FlagKind::PolarValue).once()];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-oerrexit"]).unwrap();
    assert_eq!(
        r.flags(),
        [Parsed::PolarValue('o', Polarity::On, "errexit")]
    );
    assert!(r.operands().is_empty());
}

#[test]
fn unimplemented_short_flag_is_reported_as_spelled() {
    let flags = [bool_flag('a'), bool_flag('x').unimplemented()];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-ax"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('a'), Parsed::Bool('x')]);
    assert_eq!(r.unimplemented(), [Spelling::Short('x')]);
    assert_eq!(Spelling::Short('x').to_string(), "-x");
}

#[test]
fn noop_flag_leaves_no_trace_in_the_flags() {
    let flags = [bool_flag('a'), noop_flag('f')];
    let def = scanner(&flags, false, Style::Posix, true);
    let r = def.scan(&["-fa", "-f"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('a')]);
}
