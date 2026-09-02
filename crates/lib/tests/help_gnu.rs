//! The GNU coreutils help dialect

mod fixtures;

use ecmd::meta::CommandDef;
use ecmd::parse::{FlagDef, FlagKind, OnUnknown};
use ecmd::style::{HelpStyle, Style};
use fixtures::{GNU_FLAGS, gnu_definition};

static CLAIMED_ACTION_FLAGS: [FlagDef; 2] = [
    FlagDef {
        ch: 'h',
        long: "human",
        aliases: &[],
        kind: FlagKind::Bool,
        clears: &[],
        desc: "human readable",
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
        ch: 'V',
        long: "version-sort",
        aliases: &[],
        kind: FlagKind::Bool,
        clears: &[],
        desc: "natural version sort",
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
];

#[test]
fn gnu_help_renders_usage_about_and_options() {
    let def: CommandDef = CommandDef {
        name: "basename",
        about: "Print NAME with any leading directory components removed",
        short_doc: "basename [-z] NAME [SUFFIX]",
        style: Style::Gnu,
        help_style: HelpStyle::from_parse_style(Style::Gnu),
        on_unknown: OnUnknown::Reject,
        permute: true,
        flags: &GNU_FLAGS,
        positionals: &[],
        has_rest: true,
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
    let help = def.help();
    assert!(help.starts_with("Usage: basename [-z] NAME [SUFFIX]\n"));
    assert!(help.contains("Print NAME with any leading directory components removed\n"));
    assert!(
        help.contains("  -a, --multiple\tsupport multiple arguments and treat each as a NAME\n")
    );
    assert!(help.contains("  -s, --suffix=SUFFIX\tremove a trailing SUFFIX; implies -a\n"));
    assert!(help.contains("  -h, --help\tdisplay this help and exit\n"));
    assert!(help.contains("  -V, --version\toutput version information and exit\n"));
}

#[test]
fn verbatim_help_emits_only_the_authored_block() {
    let def: CommandDef = CommandDef {
        name: "xxd",
        about: "Make a hex dump or do the reverse",
        short_doc: "xxd [options] [infile [outfile]]",
        style: Style::Gnu,
        help_style: HelpStyle::from_parse_style(Style::Gnu),
        on_unknown: OnUnknown::Reject,
        permute: true,
        flags: &GNU_FLAGS,
        positionals: &[],
        has_rest: true,
        rest_label: "",
        rest_hidden: false,
        rest_desc: "",
        rest_default: "",
        rest_required: false,
        tags: &[("verbatim_help", "")],
        description: &["ignored"],
        extra: &[
            "Usage:",
            "       xxd [options]",
            "Options:",
            "    -a  autoskip",
        ],
        exit_status: &["ignored"],
        ..CommandDef::EMPTY
    };
    assert_eq!(
        def.help(),
        "Usage:\n       xxd [options]\nOptions:\n    -a  autoskip\n"
    );
}

#[test]
fn gnu_help_footer_present_without_flags() {
    let def: CommandDef = CommandDef {
        name: "true",
        about: "do nothing, successfully",
        short_doc: "true",
        style: Style::Gnu,
        help_style: HelpStyle::from_parse_style(Style::Gnu),
        on_unknown: OnUnknown::Reject,
        permute: true,
        flags: &[],
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
    };
    let help = def.help();
    assert!(help.contains("  -h, --help\tdisplay this help and exit\n"));
    assert!(help.contains("  -V, --version\toutput version information and exit\n"));
}

#[test]
fn gnu_help_advertises_unclaimed_short_actions() {
    let definition = gnu_definition(&[]);
    let help = definition.help();
    assert!(help.contains("  -h, --help"));
    assert!(help.contains("  -V, --version"));
}

#[test]
fn gnu_help_omits_claimed_short_actions() {
    let definition = gnu_definition(&CLAIMED_ACTION_FLAGS);
    let help = definition.help();
    assert!(help.contains("      --help"));
    assert!(help.contains("      --version"));
    assert!(!help.contains("  -h, --help"));
    assert!(!help.contains("  -V, --version\t"));
}
