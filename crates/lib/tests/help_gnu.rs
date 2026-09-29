//! The GNU coreutils help dialect

use ecmd::{Def, Flag, HelpStyle, Positional, Style};
mod fixtures;

use fixtures::{gnu_definition, gnu_flags};

fn claimed_action_flags() -> Vec<Flag> {
    vec![
        Flag::new('h').long("human").desc("human readable").once(),
        Flag::new('V')
            .long("version-sort")
            .desc("natural version sort")
            .once(),
    ]
}

#[test]
fn gnu_help_renders_usage_about_and_options() {
    let def: Def = Def::builder("basename")
        .about("Print NAME with any leading directory components removed")
        .short_doc("basename [-z] NAME [SUFFIX]")
        .style(Style::Gnu)
        .help_style(HelpStyle::from_parse_style(Style::Gnu))
        .flags(gnu_flags())
        .rest(Positional::new("args").spread())
        .build();
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
    let def: Def = Def::builder("xxd")
        .about("Make a hex dump or do the reverse")
        .short_doc("xxd [options] [infile [outfile]]")
        .style(Style::Gnu)
        .help_style(HelpStyle::from_parse_style(Style::Gnu))
        .flags(gnu_flags())
        .rest(Positional::new("args").spread())
        .tag("verbatim_help", "")
        .description(["ignored"])
        .extra([
            "Usage:",
            "       xxd [options]",
            "Options:",
            "    -a  autoskip",
        ])
        .exit_status(["ignored"])
        .build();
    assert_eq!(
        def.help(),
        "Usage:\n       xxd [options]\nOptions:\n    -a  autoskip\n"
    );
}

#[test]
fn gnu_help_footer_present_without_flags() {
    let def: Def = Def::builder("true")
        .about("do nothing, successfully")
        .short_doc("true")
        .style(Style::Gnu)
        .help_style(HelpStyle::from_parse_style(Style::Gnu))
        .build();
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
    let definition = gnu_definition(&claimed_action_flags());
    let help = definition.help();
    assert!(help.contains("      --help"));
    assert!(help.contains("      --version"));
    assert!(!help.contains("  -h, --help"));
    assert!(!help.contains("  -V, --version\t"));
}

// Reproduces the first `nohup --help` usage line from GNU coreutils
#[test]
fn gnu_usage_names_the_rest_operand() {
    let def = Def::builder("nohup")
        .style(Style::Gnu)
        .positional(Positional::new("command").label("COMMAND").required())
        .rest(Positional::new("args").label("ARG"))
        .build();

    assert_eq!(def.usage(), "nohup COMMAND [ARG]...");
}

#[test]
fn gnu_usage_omits_a_hidden_rest_operand() {
    let def = Def::builder("true")
        .style(Style::Gnu)
        .rest(Positional::new("ignored").hidden())
        .build();

    assert_eq!(def.usage(), "true");
}
