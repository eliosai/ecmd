//! Short flag clusters, values, polarity and pass-through

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests verify success paths"
)]

mod fixtures;

use pretty_assertions::assert_eq;

use ecmd::error::Error;
use ecmd::parse::{FlagDef, FlagKind, OnUnknown, Parsed, scan};
use ecmd::polarity::Polarity;
use ecmd::style::Style;
use fixtures::{bool_flag, no_implicit_version_marker, noop_flag, polar_flag, value_flag};

#[test]
fn empty_args_returns_empty() {
    let r = scan::<ecmd::meta::Static>(&[], &[], OnUnknown::Reject, Style::Posix, true).unwrap();
    assert!(r.flags.is_empty());
    assert!(r.operands.is_empty());
}

#[test]
fn single_bool_flag() {
    let flags = [bool_flag('v')];
    let r = scan(&["-v"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(r.flags, [Parsed::Bool('v')]);
    assert!(r.operands.is_empty());
}

#[test]
fn bundled_bool_flags() {
    let flags = [bool_flag('a'), bool_flag('b'), bool_flag('c')];
    let r = scan(&["-abc"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(
        r.flags,
        [Parsed::Bool('a'), Parsed::Bool('b'), Parsed::Bool('c'),]
    );
}

#[test]
fn double_dash_terminates_flags() {
    let flags = [bool_flag('v')];
    let r = scan(&["--", "-v"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert!(r.flags.is_empty());
    assert_eq!(r.operands, ["-v"]);
}

#[test]
fn operands_after_flags() {
    let flags = [bool_flag('v')];
    let r = scan(
        &["-v", "file.txt"],
        &flags,
        OnUnknown::Reject,
        Style::Posix,
        true,
    )
    .unwrap();
    assert_eq!(r.flags, [Parsed::Bool('v')]);
    assert_eq!(r.operands, ["file.txt"]);
}

#[test]
fn valued_flag_separate() {
    let flags = [value_flag('o')];
    let r = scan(
        &["-o", "file"],
        &flags,
        OnUnknown::Reject,
        Style::Posix,
        true,
    )
    .unwrap();
    assert_eq!(r.flags, [Parsed::Value('o', "file".into())]);
    assert!(r.operands.is_empty());
}

#[test]
fn valued_flag_separate_with_trailing() {
    let flags = [value_flag('o')];
    let r = scan(
        &["-o", "file", "rest"],
        &flags,
        OnUnknown::Reject,
        Style::Posix,
        true,
    )
    .unwrap();
    assert_eq!(r.flags, [Parsed::Value('o', "file".into())]);
    assert_eq!(r.operands, ["rest"]);
}

#[test]
fn valued_flag_stuck() {
    let flags = [value_flag('o')];
    let r = scan(&["-ofile"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(r.flags, [Parsed::Value('o', "file".into())]);
    assert!(r.operands.is_empty());
}

#[test]
fn valued_flag_stuck_strips_equals_separator() {
    let flags = [value_flag('o')];
    let r = scan(&["-o=file"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(r.flags, [Parsed::Value('o', "file".into())]);
    assert!(r.operands.is_empty());
}

#[test]
fn valued_flag_stuck_strips_only_one_equals() {
    let flags = [value_flag('o')];
    let r = scan(&["-o==file"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(r.flags, [Parsed::Value('o', "=file".into())]);
}

#[test]
fn valued_flag_stuck_bare_equals_is_empty_value() {
    let flags = [value_flag('o')];
    let r = scan(&["-o="], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(r.flags, [Parsed::Value('o', String::new())]);
}

#[test]
fn valued_flag_missing_arg_errors() {
    let flags = [value_flag('o')];
    let r = scan(&["-o"], &flags, OnUnknown::Reject, Style::Posix, true);
    assert!(matches!(r, Err(Error::MissingValue(_))));
}

#[test]
fn bundled_bool_then_valued() {
    let flags = [bool_flag('v'), value_flag('o')];
    let r = scan(
        &["-vo", "file"],
        &flags,
        OnUnknown::Reject,
        Style::Posix,
        true,
    )
    .unwrap();
    assert_eq!(
        r.flags,
        [Parsed::Bool('v'), Parsed::Value('o', "file".into()),]
    );
    assert!(r.operands.is_empty());
}

#[test]
fn bundled_bool_then_valued_stuck() {
    let flags = [bool_flag('v'), value_flag('o')];
    let r = scan(&["-vofile"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(
        r.flags,
        [Parsed::Bool('v'), Parsed::Value('o', "file".into()),]
    );
    assert!(r.operands.is_empty());
}

#[test]
fn multiple_valued_flags_in_sequence() {
    let flags = [value_flag('o'), value_flag('d')];
    let r = scan(
        &["-o", "out", "-d", "dir"],
        &flags,
        OnUnknown::Reject,
        Style::Posix,
        true,
    )
    .unwrap();
    assert_eq!(
        r.flags,
        [
            Parsed::Value('o', "out".into()),
            Parsed::Value('d', "dir".into()),
        ]
    );
    assert!(r.operands.is_empty());
}

#[test]
fn unknown_flag_errors() {
    let flags = [bool_flag('v')];
    let r = scan(&["-x"], &flags, OnUnknown::Reject, Style::Posix, true);
    assert!(matches!(r, Err(Error::UnknownFlag(_))));
}

#[test]
fn gnu_implicit_short_help_and_version_are_actions() {
    for (arg, expected) in [
        ("-h", Error::HelpRequested),
        ("-V", Error::VersionRequested),
    ] {
        let result = scan::<ecmd::meta::Static>(&[arg], &[], OnUnknown::Reject, Style::Gnu, true);
        assert_eq!(result.unwrap_err(), expected);
    }
}

#[test]
fn gnu_reserved_actions_reject_inline_values() {
    for option in ["--help=value", "--version=value"] {
        let result =
            scan::<ecmd::meta::Static>(&[option], &[], OnUnknown::Reject, Style::Gnu, true);
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
fn no_implicit_version_marker_rejects_short_v_but_keeps_help() {
    let flags = [no_implicit_version_marker()];
    let result = scan(&["-V"], &flags, OnUnknown::Reject, Style::Gnu, true);
    assert_eq!(result.unwrap_err(), Error::UnknownFlag("-V".to_owned()));
    let result = scan(&["-h"], &flags, OnUnknown::Reject, Style::Gnu, true);
    assert_eq!(result.unwrap_err(), Error::HelpRequested);
}

#[test]
fn declared_gnu_short_help_and_version_characters_win() {
    let flags = [bool_flag('h'), bool_flag('V')];
    let result = scan(&["-hV"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(result.flags, [Parsed::Bool('h'), Parsed::Bool('V')]);
}

#[test]
fn posix_does_not_add_implicit_short_actions() {
    let result = scan::<ecmd::meta::Static>(&["-h"], &[], OnUnknown::Reject, Style::Posix, true);
    assert_eq!(result.unwrap_err(), Error::UnknownFlag("-h".to_owned()));
}

#[test]
fn passthrough_unknown_becomes_operand() {
    let flags = [bool_flag('n')];
    let r = scan(
        &["-nea"],
        &flags,
        OnUnknown::PassThrough,
        Style::Posix,
        true,
    )
    .unwrap();
    assert!(r.flags.is_empty());
    assert_eq!(r.operands, ["-nea"]);
}

#[test]
fn passthrough_all_known_still_parses() {
    let flags = [bool_flag('n'), bool_flag('e')];
    let r = scan(
        &["-ne", "hello"],
        &flags,
        OnUnknown::PassThrough,
        Style::Posix,
        true,
    )
    .unwrap();
    assert_eq!(r.flags, [Parsed::Bool('n'), Parsed::Bool('e')]);
    assert_eq!(r.operands, ["hello"]);
}

#[test]
fn passthrough_first_char_unknown() {
    let flags = [bool_flag('n')];
    let r = scan(
        &["-xyz", "rest"],
        &flags,
        OnUnknown::PassThrough,
        Style::Posix,
        true,
    )
    .unwrap();
    assert!(r.flags.is_empty());
    assert_eq!(r.operands, ["-xyz", "rest"]);
}

#[test]
fn passthrough_value_flag_stuck_still_parses() {
    let flags = [value_flag('o')];
    let r = scan(
        &["-ofile"],
        &flags,
        OnUnknown::PassThrough,
        Style::Posix,
        true,
    )
    .unwrap();
    assert_eq!(r.flags, [Parsed::Value('o', "file".into())]);
    assert!(r.operands.is_empty());
}

#[test]
fn polarity_on() {
    let flags = [polar_flag('x')];
    let r = scan(&["-x"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(r.flags, [Parsed::Polar('x', Polarity::On)]);
}

#[test]
fn polarity_off() {
    let flags = [polar_flag('x')];
    let r = scan(&["+x"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(r.flags, [Parsed::Polar('x', Polarity::Off)]);
}

#[test]
fn plus_prefix_only_when_polarity_flags_exist() {
    let flags = [bool_flag('x')];
    let r = scan(&["+x"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(r.operands, ["+x"]);
}

#[test]
fn noop_flag_accepted_silently() {
    let flags = [bool_flag('r'), noop_flag('e')];
    let r = scan(&["-re"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(r.flags, [Parsed::Bool('r')]);
}

#[test]
fn noop_flag_repetition_is_rejected_in_gnu_style() {
    let flags = [noop_flag('f')];
    let error = scan(&["-f", "-f"], &flags, OnUnknown::Reject, Style::Gnu, true)
        .expect_err("a scalar noop may occur once");
    assert_eq!(error, Error::RepeatedFlag("-f".to_owned()));
}

#[test]
fn bare_dash_is_operand() {
    let flags = [bool_flag('v')];
    let r = scan(&["-"], &flags, OnUnknown::Reject, Style::Posix, true).unwrap();
    assert_eq!(r.operands, ["-"]);
}

#[test]
fn multiple_flag_groups() {
    let flags = [bool_flag('a'), bool_flag('b')];
    let r = scan(
        &["-a", "-b", "file"],
        &flags,
        OnUnknown::Reject,
        Style::Posix,
        true,
    )
    .unwrap();
    assert_eq!(r.flags, [Parsed::Bool('a'), Parsed::Bool('b')]);
    assert_eq!(r.operands, ["file"]);
}

#[test]
fn polar_value_on() {
    let flags: [FlagDef; 1] = [FlagDef {
        ch: 'o',
        kind: FlagKind::PolarValue,
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
    }];
    let r = scan(
        &["-o", "errexit"],
        &flags,
        OnUnknown::Reject,
        Style::Posix,
        true,
    )
    .unwrap();
    assert_eq!(
        r.flags,
        [Parsed::PolarValue('o', Polarity::On, "errexit".into())]
    );
    assert!(r.operands.is_empty());
}

#[test]
fn polar_value_off() {
    let flags: [FlagDef; 1] = [FlagDef {
        ch: 'o',
        kind: FlagKind::PolarValue,
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
    }];
    let r = scan(
        &["+o", "verbose"],
        &flags,
        OnUnknown::Reject,
        Style::Posix,
        true,
    )
    .unwrap();
    assert_eq!(
        r.flags,
        [Parsed::PolarValue('o', Polarity::Off, "verbose".into())]
    );
    assert!(r.operands.is_empty());
}

#[test]
fn polar_value_stuck() {
    let flags: [FlagDef; 1] = [FlagDef {
        ch: 'o',
        kind: FlagKind::PolarValue,
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
    }];
    let r = scan(
        &["-oerrexit"],
        &flags,
        OnUnknown::Reject,
        Style::Posix,
        true,
    )
    .unwrap();
    assert_eq!(
        r.flags,
        [Parsed::PolarValue('o', Polarity::On, "errexit".into())]
    );
    assert!(r.operands.is_empty());
}
