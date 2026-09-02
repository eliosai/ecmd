//! The clap help dialects

mod fixtures;

use ecmd::meta::{CommandDef, PositionalDef};
use ecmd::parse::{FlagDef, FlagKind, OnUnknown};
use ecmd::style::{HelpStyle, Style};
use fixtures::gnu_definition;

// Reproduces the pinned `wc --help` layout from the reference build
#[test]
fn clap_help_matches_the_reference_layout() {
    static FLAGS: &[FlagDef] = &[
        FlagDef {
            ch: 'c',
            long: "bytes",
            aliases: &[],
            kind: FlagKind::Bool,
            clears: &[],
            desc: "print the byte counts",
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
            ch: '\u{e000}',
            long: "files0-from",
            aliases: &[],
            kind: FlagKind::Value,
            clears: &[],
            desc: "read input from the files specified by\nNUL-terminated names in file F",
            value_name: "F",
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
    let def: CommandDef = CommandDef {
        name: "wc",
        about: "Print newline, word, and byte counts for each FILE.",
        short_doc: "wc [OPTION]... [FILE]...",
        style: Style::Gnu,
        help_style: HelpStyle::Clap,
        on_unknown: OnUnknown::Reject,
        permute: true,
        flags: FLAGS,
        positionals: &[],
        has_rest: true,
        rest_label: "",
        rest_hidden: false,
        rest_desc: "",
        rest_default: "",
        rest_required: false,
        tags: &[],
        description: &[],
        extra: &[],
        exit_status: &[],
        ..CommandDef::EMPTY
    };

    let expected = "\
Print newline, word, and byte counts for each FILE.

Usage: wc [OPTION]... [FILE]...

Arguments:
  [args]...  

Options:
  -c, --bytes            print the byte counts
      --files0-from <F>  read input from the files specified by
                         NUL-terminated names in file F
  -h, --help             Print help
  -V, --version          Print version
";
    assert_eq!(def.help(), expected);
}

#[test]
fn clap_help_indents_every_usage_continuation() {
    let def = CommandDef {
        short_doc: "unlink FILE\nunlink OPTION",
        help_style: HelpStyle::Clap,
        ..gnu_definition(&[])
    };
    assert!(
        def.help()
            .contains("Usage: unlink FILE\n       unlink OPTION\n")
    );
}

#[test]
fn clap_help_hides_an_operand_and_notes_a_default() {
    static POSITIONALS: &[PositionalDef] = &[
        PositionalDef {
            name: "path",
            required: true,
            desc: "",
            label: "",
            default_value: "",
            hidden: true,
            spread: false,
        },
        PositionalDef {
            name: "input",
            required: false,
            desc: "",
            label: "",
            default_value: "-",
            hidden: false,
            spread: false,
        },
    ];
    let def = CommandDef {
        help_style: HelpStyle::Clap,
        positionals: POSITIONALS,
        ..gnu_definition(&[])
    };
    let help = def.help();
    assert!(!help.contains("<path>"), "hidden operand was listed");
    assert!(help.contains("  [input]  [default: -]\n"));
}

#[test]
fn clap_help_labels_the_rest_slot() {
    let def = CommandDef {
        help_style: HelpStyle::Clap,
        has_rest: true,
        rest_label: "files",
        ..gnu_definition(&[])
    };
    assert!(def.help().contains("Arguments:\n  [files]...  \n"));
}

#[test]
fn clap_help_inserts_an_arg_row_before_the_rest_slot() {
    const ARG_ROW_MODE: &[(&str, &str)] = &[("arg_row", "0\t[MODE]\t")];
    let def = CommandDef {
        help_style: HelpStyle::Clap,
        has_rest: true,
        rest_label: "FILE",
        tags: ARG_ROW_MODE,
        ..gnu_definition(&[])
    };
    assert!(
        def.help()
            .contains("Arguments:\n  [MODE]     \n  [FILE]...  \n")
    );
}

#[test]
fn clap_help_appends_an_arg_row_past_the_end_and_keeps_its_desc() {
    const ARG_ROW_SIZE: &[(&str, &str)] = &[("arg_row", "9\t[SIZE]\tbytes to keep")];
    let def = CommandDef {
        help_style: HelpStyle::Clap,
        has_rest: true,
        rest_label: "FILE",
        tags: ARG_ROW_SIZE,
        ..gnu_definition(&[])
    };
    assert!(
        def.help()
            .contains("Arguments:\n  [FILE]...  \n  [SIZE]     bytes to keep\n")
    );
}

#[test]
fn clap_help_notes_defaults_possible_values_and_aliases() {
    static FLAGS: &[FlagDef] = &[
        FlagDef {
            ch: 'q',
            long: "quiet",
            aliases: &[],
            kind: FlagKind::Bool,
            clears: &[],
            desc: "never print headers giving file names",
            value_name: "",
            hidden: false,
            implemented: true,
            repeatable: false,
            allow_hyphen_values: true,
            possible_values: &[],
            help_values: &[],
            default_value: "",
            help_label: "",
            visible_aliases: &["silent"],
        },
        FlagDef {
            ch: 'c',
            long: "color",
            aliases: &[],
            kind: FlagKind::Value,
            clears: &[],
            desc: "colorize the output",
            value_name: "WHEN",
            hidden: false,
            implemented: true,
            repeatable: false,
            allow_hyphen_values: true,
            possible_values: &["always", "auto", "never"],
            help_values: &["always", "auto", "never"],
            default_value: "auto",
            help_label: "",
            visible_aliases: &[],
        },
    ];
    let def = CommandDef {
        help_style: HelpStyle::Clap,
        flags: FLAGS,
        ..gnu_definition(&[])
    };
    let help = def.help();
    assert!(help.contains("never print headers giving file names [alias: --silent]\n"));
    assert!(
        help.contains(
            "colorize the output [default: auto] [possible values: always, auto, never]\n"
        )
    );
}
