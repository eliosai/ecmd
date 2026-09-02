//! Constructing and copying command definitions

#![expect(clippy::unwrap_used, reason = "tests verify known structure")]

mod fixtures;

use ecmd::meta::{CommandDef, OwnedCommandDef, PositionalDef};
use ecmd::parse::OnUnknown;
use ecmd::style::{HelpStyle, Style};
use fixtures::{ALIAS_FLAGS, make_def};

// ── Backward compat: existing static constructibility ──────
#[test]
fn command_def_is_const_constructible() {
    static DEF: CommandDef = CommandDef {
        name: "test",
        about: "",
        short_doc: "",
        style: Style::Posix,
        help_style: HelpStyle::from_parse_style(Style::Posix),
        on_unknown: OnUnknown::Reject,
        permute: false,
        flags: &[],
        positionals: &[PositionalDef {
            name: "target",
            required: true,
            desc: "",
            label: "",
            default_value: "",
            hidden: false,
            spread: false,
        }],
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
    };
    assert_eq!(DEF.name, "test");
    assert!(DEF.positionals.first().unwrap().required);
}

#[test]
fn owned_definition_preserves_static_shape() {
    let static_def = make_def(
        "alias",
        "Define or display aliases.",
        "",
        &ALIAS_FLAGS,
        &["Display aliases when no arguments are given."],
        &[],
        &["Returns success unless a name is invalid."],
    );

    let owned: OwnedCommandDef = static_def.clone().into_owned();

    assert_eq!(owned.name(), static_def.name);
    assert_eq!(owned.usage(), static_def.usage());
    assert_eq!(owned.help(), static_def.help());
    assert_eq!(owned.flags().len(), static_def.flags.len());
    assert_eq!(owned.scan(&["-p"]), static_def.scan(&["-p"]));
}
