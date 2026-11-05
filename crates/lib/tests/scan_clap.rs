//! The clap style, which reads POSIX and GNU argv as clap does

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "tests")]

use ecmd::Command;
use ecmd::{Error, HelpStyle, Operands, Parsed, Style};
mod fixtures;

use pretty_assertions::assert_eq;

use fixtures::{bool_flag, long_bool, long_value, noop_flag, scanner};

#[test]
fn an_attached_short_value_keeps_its_equals_under_gnu() {
    let flags = [long_value('o', "output")];
    let def = scanner(&flags, false, Style::Gnu, true);
    let r = def.scan(&["-o=v"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('o', "=v")]);
}

#[test]
fn an_attached_short_value_drops_its_equals_under_clap() {
    let flags = [long_value('o', "output")];
    let def = scanner(&flags, false, Style::Clap, true);
    let r = def.scan(&["-o=v"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('o', "v")]);
}

#[test]
fn clap_drops_only_one_leading_equals() {
    let flags = [long_value('o', "output")];
    let def = scanner(&flags, false, Style::Clap, true);
    let r = def.scan(&["-o==v"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('o', "=v")]);
}

#[test]
fn clap_keeps_a_separated_equals() {
    let flags = [long_value('o', "output")];
    let def = scanner(&flags, false, Style::Clap, true);
    let r = def.scan(&["-o", "=v"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Value('o', "=v")]);
}

#[test]
fn a_clustered_short_value_drops_its_equals_under_clap() {
    let flags = [bool_flag('a'), long_value('o', "output")];
    let def = scanner(&flags, false, Style::Clap, true);
    let r = def.scan(&["-ao=v"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('a'), Parsed::Value('o', "v")]);
}

#[test]
fn a_clustered_short_value_keeps_its_equals_under_gnu() {
    let flags = [bool_flag('a'), long_value('o', "output")];
    let def = scanner(&flags, false, Style::Gnu, true);
    let r = def.scan(&["-ao=v"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('a'), Parsed::Value('o', "=v")]);
}

#[test]
fn a_long_value_drops_its_equals_under_gnu_and_clap() {
    let flags = [long_value('o', "output")];
    for style in [Style::Gnu, Style::Clap] {
        let def = scanner(&flags, false, style, true);
        let r = def.scan(&["--output=v"]).unwrap();
        assert_eq!(r.flags(), [Parsed::Value('o', "v")]);
    }
}

#[test]
fn clap_reads_long_options_and_permutes() {
    let flags = [long_bool('a', "all")];
    let def = scanner(&flags, false, Style::Clap, true);
    let r = def.scan(&["one", "--all", "two"]).unwrap();
    assert_eq!(r.flags(), [Parsed::Bool('a')]);
    assert_eq!(r.operands(), ["one", "two"]);
}

#[test]
fn clap_refuses_a_repeated_flag() {
    let flags = [noop_flag('f')];
    let def = scanner(&flags, false, Style::Clap, true);
    let error = def
        .scan(&["-f", "-f"])
        .expect_err("a scalar flag may occur once");
    assert_eq!(error, Error::RepeatedFlag("-f".to_owned()));
}

#[derive(Command, Debug)]
#[command(
    name = "clap-optional",
    style = "clap",
    tag(optional_values = "replace=default")
)]
struct ClapOptional {
    #[flag(short = 'i')]
    replace: Option<String>,
    args: Operands,
}

#[test]
fn a_derived_clap_command_drops_the_equals_of_an_optional_short_value() {
    let short = ClapOptional::parse(&["-i=VALUE"]).unwrap();
    assert_eq!(short.replace.as_deref(), Some("VALUE"));

    let long = ClapOptional::parse(&["--replace=VALUE"]).unwrap();
    assert_eq!(long.replace.as_deref(), Some("VALUE"));

    let defaulted = ClapOptional::parse(&["-i", "operand"]).unwrap();
    assert_eq!(defaulted.replace.as_deref(), Some("default"));
    assert_eq!(&*defaulted.args, &["operand"]);
}

#[test]
fn a_derived_clap_command_parses_and_renders_as_clap() {
    let def = ClapOptional::def();
    assert_eq!(def.style(), Style::Clap);
    assert_eq!(def.help_style(), HelpStyle::Clap);
}
