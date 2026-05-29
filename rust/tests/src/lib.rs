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

    // ── Doc comment sections → bash-compatible help ────────────────

    /// Define or display aliases.
    ///
    /// Without arguments, `alias' prints the list of aliases in the reusable
    /// form `alias NAME=VALUE' on standard output.
    ///
    /// Otherwise, an alias is defined for each NAME whose VALUE is given.
    /// A trailing space in VALUE causes the next word to be checked for
    /// alias substitution when the alias is expanded.
    ///
    /// # Options
    ///
    /// # Exit Status
    /// alias returns true unless a NAME is supplied for which no alias has been
    /// defined.
    #[derive(Command)]
    #[command(name = "alias", tag(kind = "bash"), lenient,
        short_doc = "alias [-p] [name[=value] ... ]")]
    struct BashAlias {
        /// print all defined aliases in a reusable format
        #[flag(short = 'p')]
        alias_print: bool,
        alias_args: Operands,
    }

    #[test]
    fn derive_alias_help_matches_bash() {
        let expected = concat!(
            "alias: alias [-p] [name[=value] ... ]\n",
            "    Define or display aliases.\n",
            "    \n",
            "    Without arguments, `alias' prints the list of aliases in the reusable\n",
            "    form `alias NAME=VALUE' on standard output.\n",
            "    \n",
            "    Otherwise, an alias is defined for each NAME whose VALUE is given.\n",
            "    A trailing space in VALUE causes the next word to be checked for\n",
            "    alias substitution when the alias is expanded.\n",
            "    \n",
            "    Options:\n",
            "      -p\tprint all defined aliases in a reusable format\n",
            "    \n",
            "    Exit Status:\n",
            "    alias returns true unless a NAME is supplied for which no alias has been\n",
            "    defined.\n",
        );
        assert_eq!(BashAlias::def().help(), expected);
    }

    #[test]
    fn derive_alias_about_from_first_paragraph() {
        assert_eq!(BashAlias::def().about, "Define or display aliases.");
    }

    #[test]
    fn derive_alias_short_doc_in_help() {
        assert!(BashAlias::def().help().starts_with("alias: alias [-p] [name[=value] ... ]\n"));
    }

    // ── Minimal: no Options, no Exit Status ────────────────────────

    /// Exit the shell.
    ///
    /// Exits the shell with a status of N.  If N is omitted, the exit status
    /// is that of the last command executed.
    #[derive(Command)]
    #[command(name = "exit", tag(kind = "special"), tag(special),
        short_doc = "exit [n]")]
    struct BashExit {
        exit_code: Option<String>,
    }

    #[test]
    fn derive_exit_help_matches_bash() {
        let expected = concat!(
            "exit: exit [n]\n",
            "    Exit the shell.\n",
            "    \n",
            "    Exits the shell with a status of N.  If N is omitted, the exit status\n",
            "    is that of the last command executed.\n",
        );
        assert_eq!(BashExit::def().help(), expected);
    }

    // ── extra_help attribute for tab-formatted content ──────────────

    /// Write arguments to the standard output.
    ///
    /// Display the ARGs, separated by a single space character and followed by a
    /// newline, on the standard output.
    ///
    /// # Exit Status
    /// Returns success unless a write error occurs.
    #[derive(Command)]
    #[command(name = "echo2", tag(kind = "bash"), tag(no_help), lenient,
        short_doc = "echo [-neE] [arg ...]",
        extra_help(
            "Options:",
            "  -n\tdo not append a newline",
            "  -e\tenable interpretation of the following backslash escapes",
            "  -E\texplicitly suppress interpretation of backslash escapes",
        ),
    )]
    struct BashEcho {
        echo_args: Operands,
    }

    #[test]
    fn derive_echo_extra_help_replaces_doc_extra() {
        let help = BashEcho::def().help();
        assert!(help.contains("      -n\tdo not append a newline\n"));
        assert!(help.contains("    Exit Status:\n    Returns success"));
    }

    #[test]
    fn derive_echo_no_help_tag() {
        let def = BashEcho::def();
        assert!(def.tags.iter().any(|&(k, _)| k == "no_help"));
    }

    // ── Multi-line flag descriptions ───────────────────────────────

    /// Change the shell working directory.
    ///
    /// Change the current directory to DIR.
    ///
    /// # Options
    ///
    /// The default is to follow symbolic links.
    ///
    /// # Exit Status
    /// Returns 0 if the directory is changed.
    #[derive(Command)]
    #[command(name = "cd2", short_doc = "cd [-LP] [dir]")]
    struct BashCd {
        /// force symbolic links to be followed: resolve symbolic
        /// links in DIR after processing instances of `..'
        #[flag(short = 'L', clears(cd_physical))]
        cd_logical: bool,
        /// use the physical directory structure without following
        /// symbolic links
        #[flag(short = 'P', clears(cd_logical))]
        cd_physical: bool,
        cd_dir: Option<String>,
    }

    #[test]
    fn derive_cd_multiline_flag_desc() {
        let help = BashCd::def().help();
        assert!(help.contains("  -L\tforce symbolic links to be followed: resolve symbolic\n"));
        assert!(help.contains("\t\tlinks in DIR after processing instances of `..'\n"));
    }

    #[test]
    fn derive_cd_post_options_extra() {
        let help = BashCd::def().help();
        assert!(help.contains("    The default is to follow symbolic links.\n"));
    }

    #[test]
    fn derive_cd_exit_status() {
        let help = BashCd::def().help();
        assert!(help.contains("    Exit Status:\n    Returns 0 if the directory is changed.\n"));
    }

    // ── GNU style: long options, inference, permutation, help ────

    #[derive(Command)]
    #[command(name = "basename", style = "gnu", short_doc = "basename [-z] NAME [SUFFIX]")]
    struct Basename {
        /// support multiple arguments and treat each as a NAME
        #[flag(short = 'a')]
        multiple: bool,
        /// remove a trailing SUFFIX; implies -a
        #[flag(short = 's', value_name = "SUFFIX")]
        suffix: Option<String>,
        /// end each output line with NUL, not newline
        #[flag(short = 'z')]
        zero: bool,
        names: Operands,
    }

    #[test]
    fn gnu_derive_long_bool_and_operands() {
        let cmd = Basename::parse(&["--multiple", "a", "b"]).unwrap();
        assert!(cmd.multiple);
        assert_eq!(cmd.names.len(), 2);
        assert_eq!(cmd.names.get(0), Some("a"));
    }

    #[test]
    fn gnu_derive_long_value_inline() {
        let cmd = Basename::parse(&["--suffix=.txt", "x"]).unwrap();
        assert_eq!(cmd.suffix.as_deref(), Some(".txt"));
    }

    #[test]
    fn gnu_derive_long_value_separated() {
        let cmd = Basename::parse(&["--suffix", ".txt", "x"]).unwrap();
        assert_eq!(cmd.suffix.as_deref(), Some(".txt"));
    }

    #[test]
    fn gnu_derive_prefix_inference() {
        let cmd = Basename::parse(&["--mult", "x"]).unwrap();
        assert!(cmd.multiple);
    }

    #[test]
    fn gnu_derive_short_flags_still_work() {
        let cmd = Basename::parse(&["-a", "-s", ".bak", "x"]).unwrap();
        assert!(cmd.multiple);
        assert_eq!(cmd.suffix.as_deref(), Some(".bak"));
    }

    #[test]
    fn gnu_derive_permutes_flags_after_operands() {
        let cmd = Basename::parse(&["x", "--zero", "y"]).unwrap();
        assert!(cmd.zero);
        assert_eq!(cmd.names.len(), 2);
    }

    #[test]
    fn gnu_derive_help_format() {
        let help = Basename::def().help();
        assert!(help.starts_with("Usage: basename [-z] NAME [SUFFIX]\n"));
        assert!(help.contains("  -a, --multiple\tsupport multiple arguments and treat each as a NAME\n"));
        assert!(help.contains("  -s, --suffix=SUFFIX\tremove a trailing SUFFIX; implies -a\n"));
        assert!(help.contains("      --help\tdisplay this help and exit\n"));
    }

    #[test]
    fn gnu_derive_help_is_signalled() {
        let result = Basename::parse(&["--help"]);
        assert_eq!(result.err(), Some(ecmd::error::Error::HelpRequested));
    }

    // ── Long-only options (a long name with no short char) ───────

    #[derive(Command)]
    #[command(name = "paint", style = "gnu")]
    struct Paint {
        /// colorize the output
        #[flag(long = "color")]
        color: bool,
        /// be verbose
        #[flag(short = 'v')]
        verbose: bool,
        /// tint with HUE
        #[flag(long = "tint", value_name = "HUE")]
        tint: Option<String>,
        files: Operands,
    }

    #[test]
    fn long_only_flag_sets_field() {
        let cmd = Paint::parse(&["--color", "a"]).unwrap();
        assert!(cmd.color);
        assert!(!cmd.verbose);
        assert_eq!(cmd.files.len(), 1);
    }

    #[test]
    fn long_only_coexists_with_short_sibling() {
        let cmd = Paint::parse(&["-v", "--color", "x"]).unwrap();
        assert!(cmd.verbose);
        assert!(cmd.color);
    }

    #[test]
    fn long_only_prefix_inference() {
        let cmd = Paint::parse(&["--col", "x"]).unwrap();
        assert!(cmd.color);
    }

    #[test]
    fn long_only_valued_flag() {
        let cmd = Paint::parse(&["--tint=red", "x"]).unwrap();
        assert_eq!(cmd.tint.as_deref(), Some("red"));
    }

    #[test]
    fn long_only_usage_has_no_synthetic_char() {
        let usage = Paint::def().usage();
        assert!(!usage.contains('\u{E000}'), "synthetic char leaked: {usage}");
        assert!(usage.contains("[--color]"), "usage: {usage}");
        assert!(usage.contains("[--tint=HUE]"), "usage: {usage}");
        assert!(usage.contains("[-v]"), "usage: {usage}");
    }
}
