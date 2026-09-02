//! The clap help dialects

use ecmd::{Def, Flag, HelpStyle, Positional, Style};

// Reproduces the pinned `wc --help` layout from the reference build
#[test]
fn clap_help_matches_the_reference_layout() {
    fn flags() -> Vec<Flag> {
        vec![
            Flag::new('c')
                .long("bytes")
                .desc("print the byte counts")
                .once(),
            Flag::new('\u{e000}')
                .long("files0-from")
                .value("F")
                .desc("read input from the files specified by\nNUL-terminated names in file F")
                .once(),
        ]
    }
    let def: Def = Def::builder("wc")
        .about("Print newline, word, and byte counts for each FILE.")
        .short_doc("wc [OPTION]... [FILE]...")
        .style(Style::Gnu)
        .help_style(HelpStyle::Clap)
        .flags(flags())
        .rest(Positional::new("args").spread())
        .build();

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
    let def = Def::builder("")
        .short_doc("unlink FILE\nunlink OPTION")
        .help_style(HelpStyle::Clap)
        .permute(false)
        .build();
    assert!(
        def.help()
            .contains("Usage: unlink FILE\n       unlink OPTION\n")
    );
}

#[test]
fn clap_help_hides_an_operand_and_notes_a_default() {
    let def = Def::builder("")
        .help_style(HelpStyle::Clap)
        .permute(false)
        .positional(Positional::new("path").required().hidden())
        .positional(Positional::new("input").default_value("-"))
        .build();
    let help = def.help();
    assert!(!help.contains("<path>"), "hidden operand was listed");
    assert!(help.contains("  [input]  [default: -]\n"));
}

#[test]
fn clap_help_labels_the_rest_slot() {
    let def = Def::builder("")
        .help_style(HelpStyle::Clap)
        .permute(false)
        .rest(Positional::new("files").spread())
        .build();
    assert!(def.help().contains("Arguments:\n  [files]...  \n"));
}

#[test]
fn clap_help_inserts_an_arg_row_before_the_rest_slot() {
    const ARG_ROW_MODE: &[(&str, &str)] = &[("arg_row", "0\t[MODE]\t")];
    let def = Def::builder("")
        .help_style(HelpStyle::Clap)
        .permute(false)
        .rest(Positional::new("FILE").spread())
        .tags(ARG_ROW_MODE.iter().copied())
        .build();
    assert!(
        def.help()
            .contains("Arguments:\n  [MODE]     \n  [FILE]...  \n")
    );
}

#[test]
fn clap_help_appends_an_arg_row_past_the_end_and_keeps_its_desc() {
    const ARG_ROW_SIZE: &[(&str, &str)] = &[("arg_row", "9\t[SIZE]\tbytes to keep")];
    let def = Def::builder("")
        .help_style(HelpStyle::Clap)
        .permute(false)
        .rest(Positional::new("FILE").spread())
        .tags(ARG_ROW_SIZE.iter().copied())
        .build();
    assert!(
        def.help()
            .contains("Arguments:\n  [FILE]...  \n  [SIZE]     bytes to keep\n")
    );
}

#[test]
fn clap_help_notes_defaults_possible_values_and_aliases() {
    fn flags() -> Vec<Flag> {
        vec![
            Flag::new('q')
                .long("quiet")
                .desc("never print headers giving file names")
                .once()
                .visible_alias("silent"),
            Flag::new('c')
                .long("color")
                .value("WHEN")
                .desc("colorize the output")
                .once()
                .possible_values(["always", "auto", "never"])
                .help_values(["always", "auto", "never"])
                .default_value("auto"),
        ]
    }
    let def = Def::builder("")
        .help_style(HelpStyle::Clap)
        .permute(false)
        .flags(flags())
        .build();
    let help = def.help();
    assert!(help.contains("never print headers giving file names [alias: --silent]\n"));
    assert!(
        help.contains(
            "colorize the output [default: auto] [possible values: always, auto, never]\n"
        )
    );
}
