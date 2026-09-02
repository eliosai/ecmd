//! Building definitions at runtime and reading them back

#![expect(clippy::unwrap_used, reason = "tests verify known structure")]

use ecmd::{Command, Def, Flag, Positional, Style};

/// A sample command.
#[derive(Command)]
#[command(name = "sample", style = "gnu")]
struct Sample {
    /// list everything
    #[flag(short = 'a', long = "all")]
    all: bool,
    /// the output file
    #[flag(short = 'o', long = "out", value_name = "FILE")]
    out: Option<String>,
    source: String,
    rest: ecmd::Operands,
}

fn built() -> Def {
    Def::builder("sample")
        .about("A sample command.")
        .style(Style::Gnu)
        .flag(Flag::new('a').long("all").desc("list everything"))
        .flag(
            Flag::new('o')
                .long("out")
                .value("FILE")
                .desc("the output file"),
        )
        .positional(Positional::new("source").required())
        .rest(Positional::new("rest"))
        .build()
}

#[test]
fn the_derived_command_reads_every_field() {
    let sample = Sample::parse(&["-a", "-o", "out.txt", "src", "more"]).unwrap();
    assert!(sample.all);
    assert_eq!(sample.out.as_deref(), Some("out.txt"));
    assert_eq!(sample.source, "src");
    assert_eq!(sample.rest.first(), Some("more"));
}

#[test]
fn the_builder_matches_the_derived_definition() {
    assert_eq!(&built(), Sample::def());
    assert_eq!(built().help(), Sample::def().help());
}

#[test]
fn a_built_definition_scans_like_the_derived_one() {
    let def = built();
    let scan = def.scan(&["-a", "--out=x", "src", "extra"]).unwrap();
    let derived = Sample::def()
        .scan(&["-a", "--out=x", "src", "extra"])
        .unwrap();
    assert_eq!(scan, derived);
    assert_eq!(scan.operands(), ["src", "extra"]);
}

#[test]
fn a_long_only_flag_takes_a_synthetic_identity_from_its_position() {
    let def = Def::builder("t")
        .flag(Flag::new('v'))
        .flag(Flag::long_only("verbose"))
        .build();
    let verbose = def.flag_named("verbose").unwrap();
    assert_eq!(verbose.short(), None);
    assert_eq!(verbose.long_name(), Some("verbose"));
    assert!(verbose.id() >= '\u{E000}');
    assert_eq!(def.flag(verbose.id()).map(Flag::id), Some(verbose.id()));
    assert_eq!(def.flag_named("v").map(Flag::id), Some('v'));
}

#[test]
fn a_definition_can_be_renamed_and_tagged() {
    let def = built().with_name("alias").with_tag("special", "");
    assert_eq!(def.name(), "alias");
    assert_eq!(def.tag("special"), Some(""));
    assert_eq!(def.tags().collect::<Vec<_>>(), [("special", "")]);
    assert!(def.usage().starts_with("alias "));
}

#[test]
fn a_flag_refuses_values_outside_its_accepted_set() {
    let def = Def::builder("t")
        .flag(
            Flag::new('m')
                .value("MODE")
                .possible_values(["fast", "slow"]),
        )
        .build();
    assert_eq!(
        def.scan(&["-m", "fast"]).unwrap().flags(),
        [ecmd::Parsed::Value('m', "fast")]
    );
    assert_eq!(
        def.scan(&["-m", "warp"]).unwrap_err(),
        ecmd::Error::InvalidValue {
            flag: "-m".to_owned(),
            value: "warp".to_owned(),
            reason: "expected one of fast, slow".to_owned(),
        }
    );
}
