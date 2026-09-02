//! The util-linux help dialect

use ecmd::meta::CommandDef;
use ecmd::parse::{FlagDef, FlagKind};
use ecmd::style::{HelpStyle, Style};

static UTIL_LINUX_FLAGS: [FlagDef; 3] = [
    FlagDef {
        ch: 's',
        long: "single-shot",
        desc: "return one PID only",
        implemented: true,
        ..FlagDef::EMPTY
    },
    FlagDef {
        ch: 'o',
        long: "omit-pid",
        kind: FlagKind::Value,
        value_name: "PID,...",
        desc: "omit processes with PID",
        implemented: true,
        ..FlagDef::EMPTY
    },
    FlagDef {
        ch: '\u{e000}',
        long: "wide",
        desc: "wide output",
        implemented: true,
        ..FlagDef::EMPTY
    },
];

fn util_linux_def(tags: &'static [(&'static str, &'static str)]) -> CommandDef {
    CommandDef {
        name: "pidof",
        about: "Find the process ID of a running program",
        short_doc: "pidof [options] [program [...]]",
        style: Style::Gnu,
        help_style: HelpStyle::UtilLinux,
        permute: true,
        flags: &UTIL_LINUX_FLAGS,
        has_rest: true,
        tags,
        ..CommandDef::EMPTY
    }
}

#[test]
fn util_linux_help_injects_a_literal_row_at_its_index() {
    let help = util_linux_def(&[
        ("help_width", "27"),
        (
            "help_row",
            "0\t-<sig>\tsignal to send (either number or name)",
        ),
    ])
    .help();
    assert!(help.contains(
        "Options:\n -<sig>                    signal to send (either number or name)\n -s, --single-shot"
    ));
}

#[test]
fn util_linux_help_hangs_a_literal_row_whose_description_opens_blank() {
    let help = util_linux_def(&[
        ("help_width", "27"),
        (
            "help_row",
            "0\t-s, --input-separator, --separator <string>\t\npossible table delimiters",
        ),
    ])
    .help();
    assert!(help.contains(
        "Options:\n -s, --input-separator, --separator <string>\n                             possible table delimiters\n"
    ));
}

#[test]
fn util_linux_help_separates_rows_and_appends_extra_blocks() {
    static EXTRA: [&str; 1] = ["Arguments:\n Values for <length> may carry a suffix."];
    let mut def = util_linux_def(&[("help_row", "1\t\t")]);
    def.extra = &EXTRA;
    let help = def.help();
    assert!(help.contains("return one PID only\n\n -o, --omit-pid"));
    assert!(
        help.contains(
            "\n\nArguments:\n Values for <length> may carry a suffix.\n\nFor more details"
        )
    );
}

#[test]
fn util_linux_help_opens_blank_and_closes_with_the_manpage() {
    let help = util_linux_def(&[]).help();
    assert!(help.starts_with("\nUsage:\n pidof [options] [program [...]]\n\nOptions:\n"));
    assert!(help.ends_with("\nFor more details see pidof(1).\n"));
}

#[test]
fn util_linux_help_places_long_only_flags_under_the_long_column() {
    let help = util_linux_def(&[("help_width", "27")]).help();
    assert!(help.contains("\n -s, --single-shot         return one PID only\n"));
    assert!(help.contains("\n -o, --omit-pid <PID,...>  omit processes with PID\n"));
    assert!(help.contains("\n     --wide                wide output\n"));
}

#[test]
fn util_linux_help_gives_the_help_pair_its_own_column() {
    let help = util_linux_def(&[("help_width", "27"), ("help_pair_width", "16")]).help();
    assert!(help.contains("\n -h, --help     display this help and exit\n"));
    assert!(help.contains("\n -V, --version  output version information and exit\n"));
}

#[test]
fn util_linux_help_spaces_the_pair_and_renames_the_manpage_on_request() {
    let help = util_linux_def(&[("help_spaced", "1"), ("man_page", "pgrep")]).help();
    assert!(help.contains("wide output\n\n -h, --help"));
    assert!(help.ends_with("For more details see pgrep(1).\n"));
}

#[test]
fn util_linux_help_drops_the_trailer_when_the_reference_has_none() {
    let help = util_linux_def(&[("no_trailer", "1")]).help();
    assert!(!help.contains("For more details"));
    assert!(help.ends_with("output version information and exit\n"));
}
