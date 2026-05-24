//! Integration tests for ecmd derive macro.

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests")]
mod tests {
    use ecmd::Command;
    use ecmd::meta::Command as CommandTrait;
    use ecmd::operands::Operands;
    use ecmd::polarity::{PolarVal, Polarity};

    // ── Basic: bool flags + positionals ─────────────────────────

    #[derive(Command)]
    #[command(name = "cd")]
    struct Cd {
        #[flag(short = 'L', clears(physical))]
        logical: bool,
        #[flag(short = 'P', clears(logical))]
        physical: bool,
        dir: Option<String>,
    }

    #[test]
    fn cd_no_args() {
        let cmd = Cd::parse(&[]).unwrap();
        assert!(!cmd.logical);
        assert!(!cmd.physical);
        assert_eq!(cmd.dir, None);
    }

    #[test]
    fn cd_with_dir() {
        let cmd = Cd::parse(&["/tmp"]).unwrap();
        assert_eq!(cmd.dir, Some("/tmp".to_owned()));
    }

    #[test]
    fn cd_physical_flag() {
        let cmd = Cd::parse(&["-P", "/home"]).unwrap();
        assert!(cmd.physical);
        assert!(!cmd.logical);
        assert_eq!(cmd.dir, Some("/home".to_owned()));
    }

    #[test]
    fn cd_last_wins_mutex() {
        let cmd = Cd::parse(&["-L", "-P"]).unwrap();
        assert!(cmd.physical);
        assert!(!cmd.logical);
    }

    // ── Lenient mode ────────────────────────────────────────────

    #[derive(Command)]
    #[command(name = "echo", lenient)]
    struct Echo {
        #[flag(short = 'n')]
        no_newline: bool,
        #[flag(short = 'e')]
        escapes: bool,
        #[flag(short = 'E')]
        no_escapes: bool,
        args: Operands,
    }

    #[test]
    fn echo_simple() {
        let cmd = Echo::parse(&["hello", "world"]).unwrap();
        assert!(!cmd.no_newline);
        assert!(!cmd.escapes);
        assert!(!cmd.no_escapes);
        assert_eq!(cmd.args.join(" "), "hello world");
    }

    #[test]
    fn echo_with_flag() {
        let cmd = Echo::parse(&["-n", "hello"]).unwrap();
        assert!(cmd.no_newline);
        assert_eq!(cmd.args.join(" "), "hello");
    }

    #[test]
    fn echo_lenient_unknown_becomes_operand() {
        let cmd = Echo::parse(&["-nXYZ", "rest"]).unwrap();
        assert!(!cmd.no_newline);
        assert_eq!(cmd.args.join(" "), "-nXYZ rest");
    }

    // ── Polarity ────────────────────────────────────────────────

    #[derive(Command)]
    #[command(name = "declare")]
    struct Declare {
        #[flag(short = 'a')]
        array: Polarity,
        #[flag(short = 'x')]
        export: Polarity,
        #[flag(short = 'p')]
        print: bool,
        args: Operands,
    }

    #[test]
    fn declare_polarity_on() {
        let cmd = Declare::parse(&["-ax", "foo"]).unwrap();
        assert_eq!(cmd.array, Polarity::On);
        assert_eq!(cmd.export, Polarity::On);
        assert!(!cmd.print);
        assert_eq!(cmd.args.first(), Some("foo"));
    }

    #[test]
    fn declare_print_flag() {
        let cmd = Declare::parse(&["-p"]).unwrap();
        assert!(cmd.print);
        assert_eq!(cmd.array, Polarity::Unset);
    }

    #[test]
    fn declare_polarity_off() {
        let cmd = Declare::parse(&["+a", "bar"]).unwrap();
        assert_eq!(cmd.array, Polarity::Off);
        assert_eq!(cmd.args.first(), Some("bar"));
    }

    // ── Phase 1a: Repeatable valued flags ───────────────────────

    #[derive(Command)]
    #[command(name = "hash")]
    struct Hash {
        #[flag(short = 'd')]
        delete: bool,
        #[flag(short = 'p')]
        paths: Vec<String>,
        args: Operands,
    }

    #[test]
    fn hash_repeatable_single() {
        let cmd = Hash::parse(&["-p", "/usr/bin", "ls"]).unwrap();
        assert_eq!(cmd.paths, vec!["/usr/bin"]);
        assert_eq!(cmd.args.first(), Some("ls"));
    }

    #[test]
    fn hash_repeatable_multiple() {
        let cmd = Hash::parse(&["-p", "/a", "-p", "/b"]).unwrap();
        assert_eq!(cmd.paths, vec!["/a", "/b"]);
    }

    #[test]
    fn hash_repeatable_empty() {
        let cmd = Hash::parse(&["-d"]).unwrap();
        assert!(cmd.delete);
        assert!(cmd.paths.is_empty());
    }

    // ── Phase 1a+: Repeatable parsed (Vec<u32>) ─────────────────

    #[derive(Command)]
    #[command(name = "ulimit")]
    struct Ulimit {
        #[flag(short = 'n')]
        limits: Vec<u32>,
    }

    #[test]
    fn ulimit_repeatable_parsed() {
        let cmd = Ulimit::parse(&["-n", "1024", "-n", "4096"]).unwrap();
        assert_eq!(cmd.limits, vec![1024_u32, 4096]);
    }

    #[test]
    fn ulimit_repeatable_parsed_error() {
        let result = Ulimit::parse(&["-n", "abc"]);
        assert!(result.is_err());
    }

    // ── Phase 1b: FromStr parsing ───────────────────────────────

    #[derive(Command)]
    #[command(name = "read", noop = "eE")]
    struct ReadCmd {
        #[flag(short = 'r')]
        raw: bool,
        #[flag(short = 'n')]
        nchars: Option<usize>,
        #[flag(short = 't')]
        timeout: Option<f64>,
        #[flag(short = 'p')]
        prompt: Option<String>,
        vars: Operands,
    }

    #[test]
    fn read_typed_usize() {
        let cmd = ReadCmd::parse(&["-n", "5", "var"]).unwrap();
        assert_eq!(cmd.nchars, Some(5));
        assert_eq!(cmd.vars.first(), Some("var"));
    }

    #[test]
    fn read_typed_float() {
        let cmd = ReadCmd::parse(&["-t", "1.5"]).unwrap();
        assert_eq!(cmd.timeout, Some(1.5));
    }

    #[test]
    fn read_typed_string_still_works() {
        let cmd = ReadCmd::parse(&["-p", "Enter: "]).unwrap();
        assert_eq!(cmd.prompt, Some("Enter: ".to_owned()));
    }

    #[test]
    fn read_typed_invalid_errors() {
        let result = ReadCmd::parse(&["-n", "abc"]);
        assert!(result.is_err());
    }

    // ── Phase 1c: PolarValue (set -o name) ──────────────────────

    #[derive(Command)]
    #[command(name = "set")]
    struct Set {
        #[flag(short = 'e')]
        errexit: Polarity,
        #[flag(short = 'x')]
        xtrace: Polarity,
        #[flag(short = 'o')]
        option: Vec<PolarVal>,
        positional: Operands,
    }

    #[test]
    fn set_polar_value_on() {
        let cmd = Set::parse(&["-o", "errexit"]).unwrap();
        assert_eq!(cmd.option.len(), 1);
        assert_eq!(cmd.option[0].polarity, Polarity::On);
        assert_eq!(cmd.option[0].value, "errexit");
    }

    #[test]
    fn set_polar_value_off() {
        let cmd = Set::parse(&["+o", "verbose"]).unwrap();
        assert_eq!(cmd.option[0].polarity, Polarity::Off);
        assert_eq!(cmd.option[0].value, "verbose");
    }

    #[test]
    fn set_mixed_flags_and_polar() {
        let cmd = Set::parse(&["-ex", "-o", "pipefail"]).unwrap();
        assert_eq!(cmd.errexit, Polarity::On);
        assert_eq!(cmd.xtrace, Polarity::On);
        assert_eq!(cmd.option[0].value, "pipefail");
    }

    // ── Required positional ─────────────────────────────────────

    #[derive(Command)]
    #[command(name = "grep")]
    struct Grep {
        #[flag(short = 'i')]
        ignore_case: bool,
        #[flag(short = 'n')]
        line_numbers: bool,
        pattern: String,
        files: Operands,
    }

    #[test]
    fn grep_with_pattern_and_files() {
        let cmd = Grep::parse(&["-i", "hello", "a.rs", "b.rs"]).unwrap();
        assert!(cmd.ignore_case);
        assert_eq!(cmd.pattern, "hello");
        assert_eq!(cmd.files.len(), 2);
    }

    #[test]
    fn grep_missing_pattern_errors() {
        let result = Grep::parse(&["-i"]);
        assert!(result.is_err());
    }

    // ── Edge cases ──────────────────────────────────────────────

    #[test]
    fn grep_double_dash_with_positional() {
        let cmd = Grep::parse(&["--", "-i", "file"]).unwrap();
        assert!(!cmd.ignore_case);
        assert_eq!(cmd.pattern, "-i");
        assert_eq!(cmd.files.first(), Some("file"));
    }

    #[test]
    fn read_bundled_with_fromstr_value() {
        let cmd = ReadCmd::parse(&["-rn5"]).unwrap();
        assert!(cmd.raw);
        assert_eq!(cmd.nchars, Some(5));
    }

    #[test]
    fn plus_prefix_with_only_bool_flags_is_operand() {
        let cmd = Grep::parse(&["+i", "file"]).unwrap();
        assert!(!cmd.ignore_case);
        assert_eq!(cmd.pattern, "+i");
        assert_eq!(cmd.files.first(), Some("file"));
    }

    #[test]
    fn cd_def_has_flags() {
        let def = Cd::def();
        assert_eq!(def.name, "cd");
        assert!(!def.flags.is_empty());
        assert!(def.has_rest == false);
    }

    #[test]
    fn grep_def_has_positionals() {
        let def = Grep::def();
        assert_eq!(def.positionals.len(), 1);
        assert!(def.positionals[0].required);
        assert!(def.has_rest);
    }

    // ── Doc comments → about/desc ──────────────────────────────────

    /// Change the shell working directory.
    #[derive(Command)]
    #[command(name = "mycd")]
    struct MyCd {
        /// Follow symlinks (default).
        #[flag(short = 'L', clears(my_physical))]
        my_logical: bool,
        /// Use physical directory.
        #[flag(short = 'P', clears(my_logical))]
        my_physical: bool,
        /// Target directory.
        target: Option<String>,
    }

    #[test]
    fn doc_comment_about() {
        assert_eq!(MyCd::def().about, "Change the shell working directory.");
    }

    #[test]
    fn doc_comment_flag_desc() {
        let flags = MyCd::def().flags;
        assert_eq!(flags[0].desc, "Follow symlinks (default).");
        assert_eq!(flags[1].desc, "Use physical directory.");
    }

    #[test]
    fn doc_comment_positional_desc() {
        let pos = MyCd::def().positionals;
        assert_eq!(pos[0].desc, "Target directory.");
    }

    #[test]
    fn usage_auto_generated() {
        let usage = MyCd::def().usage();
        assert_eq!(usage, "mycd [-LP] [target]");
    }

    #[test]
    fn help_contains_about_and_flags() {
        let help = MyCd::def().help();
        assert!(help.contains("Change the shell working directory."));
        assert!(help.contains("-L"));
        assert!(help.contains("Follow symlinks"));
        assert!(help.contains("-P"));
        assert!(help.contains("Use physical directory"));
    }

    // ── value_name ─────────────────────────────────────────────────

    /// Read input.
    #[derive(Command)]
    #[command(name = "myread", noop = "eE")]
    struct MyRead {
        /// Raw mode.
        #[flag(short = 'r')]
        raw: bool,
        /// Read N characters.
        #[flag(short = 'n', value_name = "NCHARS")]
        nchars: Option<usize>,
        /// Prompt string.
        #[flag(short = 'p', value_name = "PROMPT")]
        prompt: Option<String>,
        vars: Operands,
    }

    #[test]
    fn value_name_in_usage() {
        let usage = MyRead::def().usage();
        assert!(usage.contains("[-n NCHARS]"), "got: {usage}");
        assert!(usage.contains("[-p PROMPT]"), "got: {usage}");
    }

    #[test]
    fn value_name_in_help() {
        let help = MyRead::def().help();
        assert!(help.contains("-n NCHARS"), "got: {help}");
        assert!(help.contains("Read N characters"), "got: {help}");
    }

    #[test]
    fn value_name_in_flag_def() {
        let flags = MyRead::def().flags;
        let n_flag = flags.iter().find(|f| f.ch == 'n').unwrap();
        assert_eq!(n_flag.value_name, "NCHARS");
    }

    // ── Tags ───────────────────────────────────────────────────────

    /// Export variables.
    #[derive(Command)]
    #[command(name = "export", tag(kind = "special"), tag(special), tag(assignment))]
    struct Export {
        #[flag(short = 'n')]
        unexport: bool,
        #[flag(short = 'p')]
        print: bool,
        names: Operands,
    }

    #[test]
    fn tags_present() {
        let def = Export::def();
        assert_eq!(def.tags.len(), 3);
        assert_eq!(def.tags[0], ("kind", "special"));
        assert_eq!(def.tags[1], ("special", ""));
        assert_eq!(def.tags[2], ("assignment", ""));
    }

    #[test]
    fn tags_empty_when_none() {
        assert!(MyCd::def().tags.is_empty());
    }
}
