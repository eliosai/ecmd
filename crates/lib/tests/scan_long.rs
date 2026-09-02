//! GNU long options, prefix inference and permutation

#![expect(clippy::unwrap_used, reason = "tests verify success paths")]

mod fixtures;

use pretty_assertions::assert_eq;

use ecmd::error::Error;
use ecmd::parse::{FlagDef, FlagKind, OnUnknown, Parsed, Spelling, scan};
use ecmd::style::Style;
use fixtures::{aliased_bool, long_bool, long_value};

#[test]
fn gnu_exact_long_bool() {
    let flags = [long_bool('a', "multiple")];
    let r = scan(&["--multiple"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('a')]);
    assert!(r.operands().is_empty());
}

#[test]
fn gnu_long_value_inline() {
    let flags = [long_value('s', "suffix")];
    let r = scan(
        &["--suffix=.txt"],
        &flags,
        OnUnknown::Reject,
        Style::Gnu,
        true,
    )
    .unwrap();
    assert_eq!(r.flags(), [Parsed::Value('s', ".txt")]);
}

#[test]
fn gnu_long_value_separated() {
    let flags = [long_value('s', "suffix")];
    let r = scan(
        &["--suffix", ".txt"],
        &flags,
        OnUnknown::Reject,
        Style::Gnu,
        true,
    )
    .unwrap();
    assert_eq!(r.flags(), [Parsed::Value('s', ".txt")]);
}

#[test]
fn gnu_empty_inline_value_is_allowed() {
    let flags = [long_value('s', "suffix")];
    let r = scan(&["--suffix="], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('s', "")]);
}

#[test]
fn gnu_prefix_inference_resolves_unambiguous() {
    let flags = [long_bool('a', "multiple")];
    let r = scan(&["--mult"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('a')]);
}

#[test]
fn gnu_prefix_ambiguous_is_error() {
    let flags = [long_bool('v', "verbose")];
    // "--ver" matches both "verbose" and the reserved "version".
    let e = scan(&["--ver"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap_err();
    assert_eq!(e, Error::AmbiguousOption("--ver".into()));
}

#[test]
fn gnu_exact_match_beats_prefix() {
    let flags = [long_bool('a', "no"), long_bool('b', "nonsense")];
    let r = scan(&["--no"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('a')]);
}

#[test]
fn gnu_bool_rejects_inline_value() {
    let flags = [long_bool('a', "multiple")];
    let e = scan(
        &["--multiple=x"],
        &flags,
        OnUnknown::Reject,
        Style::Gnu,
        true,
    )
    .unwrap_err();
    assert_eq!(
        e,
        Error::UnexpectedValue {
            flag: "--multiple".into(),
            value: "x".into(),
        }
    );
}

#[test]
fn gnu_permutes_flags_after_operands() {
    let flags = [long_bool('a', "multiple"), long_bool('z', "zero")];
    let r = scan(
        &["one", "--multiple", "two", "--zero"],
        &flags,
        OnUnknown::Reject,
        Style::Gnu,
        true,
    )
    .unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('a'), Parsed::Bool('z')]);
    assert_eq!(r.operands(), ["one", "two"]);
}

#[test]
fn gnu_short_flags_still_bundle() {
    let flags = [long_bool('a', "multiple"), long_bool('z', "zero")];
    let r = scan(&["-az"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('a'), Parsed::Bool('z')]);
}

#[test]
fn gnu_short_value_separated_still_works() {
    let flags = [long_value('s', "suffix")];
    let r = scan(&["-s", ".bak"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('s', ".bak")]);
}

#[test]
fn gnu_help_is_signalled() {
    let e = scan::<ecmd::meta::Static>(&["--help"], &[], OnUnknown::Reject, Style::Gnu, true)
        .unwrap_err();
    assert_eq!(e, Error::HelpRequested);
}

#[test]
fn gnu_version_is_signalled() {
    let e = scan::<ecmd::meta::Static>(&["--version"], &[], OnUnknown::Reject, Style::Gnu, true)
        .unwrap_err();
    assert_eq!(e, Error::VersionRequested);
}

#[test]
fn gnu_help_prefix_infers() {
    let e = scan::<ecmd::meta::Static>(&["--hel"], &[], OnUnknown::Reject, Style::Gnu, true)
        .unwrap_err();
    assert_eq!(e, Error::HelpRequested);
}

#[test]
fn gnu_unknown_long_is_error() {
    let e = scan::<ecmd::meta::Static>(&["--bogus"], &[], OnUnknown::Reject, Style::Gnu, true)
        .unwrap_err();
    assert_eq!(e, Error::UnknownFlag("--bogus".into()));
}

#[test]
fn gnu_long_value_missing_errors() {
    let flags = [long_value('s', "suffix")];
    let e = scan(&["--suffix"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap_err();
    assert_eq!(e, Error::MissingValue("--suffix".into()));
}

#[test]
fn gnu_empty_long_name_is_unknown() {
    let e = scan::<ecmd::meta::Static>(&["--=x"], &[], OnUnknown::Reject, Style::Gnu, true)
        .unwrap_err();
    assert_eq!(e, Error::UnknownFlag("--".into()));
}

#[test]
fn gnu_value_flag_swallows_optionlike_next() {
    // getopt_long parity: --suffix consumes the following --zero as its value.
    let flags = [long_value('s', "suffix"), long_bool('z', "zero")];
    let r = scan(
        &["--suffix", "--zero"],
        &flags,
        OnUnknown::Reject,
        Style::Gnu,
        true,
    )
    .unwrap();
    assert_eq!(r.flags(), [Parsed::Value('s', "--zero")]);
    assert!(r.operands().is_empty());
}

#[test]
fn gnu_real_help_flag_shadows_reserved() {
    let flags = [long_bool('h', "help")];
    let r = scan(&["--help"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('h')]);
    // `--hel` is not ambiguous: the declared "help" flag shadows the reserved signal.
    let r2 = scan(&["--hel"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r2.flags(), [Parsed::Bool('h')]);
}

#[test]
fn gnu_exact_valued_beats_prefix() {
    let flags = [long_value('a', "max"), long_value('b', "maximum")];
    let r = scan(&["--max=5"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('a', "5")]);
}

#[test]
fn gnu_double_dash_still_terminates() {
    let flags = [long_bool('a', "multiple")];
    let r = scan(
        &["--", "--multiple"],
        &flags,
        OnUnknown::Reject,
        Style::Gnu,
        true,
    )
    .unwrap();
    assert!(r.flags().is_empty());
    assert_eq!(r.operands(), ["--multiple"]);
}

#[test]
fn gnu_no_permute_stops_at_first_operand() {
    // with permute off the first operand ends option scanning, so later flags are operands
    let flags = [long_bool('z', "zero")];
    let r = scan(
        &["foo", "-z", "--zero"],
        &flags,
        OnUnknown::Reject,
        Style::Gnu,
        false,
    )
    .unwrap();
    assert!(r.flags().is_empty());
    assert_eq!(r.operands(), ["foo", "-z", "--zero"]);
}

#[test]
fn gnu_permute_still_collects_flags_after_operands() {
    // permute=true keeps GNU permutation (flags recognized after operands).
    let flags = [long_bool('z', "zero")];
    let r = scan(&["foo", "-z"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('z')]);
    assert_eq!(r.operands(), ["foo"]);
}

#[test]
fn posix_never_parses_double_dash_word_as_long() {
    // Bash/POSIX mode must not gain GNU long-option behavior.
    let flags = [long_bool('a', "multiple")];
    let e = scan(
        &["--multiple"],
        &flags,
        OnUnknown::Reject,
        Style::Posix,
        true,
    )
    .unwrap_err();
    assert_eq!(e, Error::UnknownFlag("--".into()));
}

#[test]
fn gnu_alias_resolves_to_flag() {
    let flags = [aliased_bool('q', "quiet", &["silent"])];
    let r = scan(&["--silent"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('q')]);
}

#[test]
fn gnu_alias_prefix_infers() {
    let flags = [aliased_bool('q', "quiet", &["silent"])];
    let r = scan(&["--si"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('q')]);
    let r2 = scan(&["--s"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r2.flags(), [Parsed::Bool('q')]);
}

#[test]
fn gnu_alias_and_long_prefix_counts_flag_once() {
    // the long "quiet" owns the "silent" alias, so "--qu" must not read as ambiguous
    let flags = [aliased_bool('q', "quiet", &["silent"])];
    let r = scan(&["--qu"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('q')]);
}

#[test]
fn gnu_hidden_flag_still_parses() {
    let flags: [FlagDef; 1] = [FlagDef {
        ch: '\u{E000}',
        long: "presume-input-pipe",
        aliases: &[],
        kind: FlagKind::Bool,
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
    }];
    let r = scan(
        &["--presume-input-pipe"],
        &flags,
        OnUnknown::Reject,
        Style::Gnu,
        true,
    )
    .unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('\u{E000}')]);
}

#[test]
fn gnu_visible_alias_matches_exactly_and_by_prefix() {
    let flags = [FlagDef {
        visible_aliases: &["quiet"][..],
        ..long_bool('s', "silent")
    }];
    let exact = scan(&["--quiet"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(exact.flags(), [Parsed::Bool('s')]);
    let prefix = scan(&["--qui"], &flags, OnUnknown::Reject, Style::Gnu, true).unwrap();
    assert_eq!(prefix.flags(), [Parsed::Bool('s')]);
}

#[test]
fn gnu_unimplemented_long_flag_is_reported_as_spelled() {
    let flags = [FlagDef {
        implemented: false,
        ..long_value('p', "ftp-port")
    }];
    let r = scan(
        &["--ftp-port=x"],
        &flags,
        OnUnknown::Reject,
        Style::Gnu,
        true,
    )
    .unwrap();
    assert_eq!(r.flags(), [Parsed::Value('p', "x")]);
    assert_eq!(r.unimplemented(), [Spelling::Long("ftp-port")]);
    assert_eq!(Spelling::Long("ftp-port").to_string(), "--ftp-port");
}
