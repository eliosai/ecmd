//! The bash builtin help dialect against real `help` output

use ecmd::{Def, Flag, Positional};
mod fixtures;

use fixtures::{alias_flags, make_def};

#[test]
fn help_alias_matches_bash() {
    let def = make_def(
        "alias",
        "Define or display aliases.",
        "alias [-p] [name[=value] ... ]",
        &alias_flags(),
        &[
            "Without arguments, `alias' prints the list of aliases in the reusable",
            "form `alias NAME=VALUE' on standard output.",
            "",
            "Otherwise, an alias is defined for each NAME whose VALUE is given.",
            "A trailing space in VALUE causes the next word to be checked for",
            "alias substitution when the alias is expanded.",
        ],
        &[],
        &[
            "alias returns true unless a NAME is supplied for which no alias has been",
            "defined.",
        ],
    );

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
    assert_eq!(def.help(), expected);
}

// ── Real `bash -c 'help exit'` output: no Options and no Exit Status ─────────
#[test]
fn help_exit_matches_bash() {
    let def = make_def(
        "exit",
        "Exit the shell.",
        "exit [n]",
        &[],
        &[
            "Exits the shell with a status of N.  If N is omitted, the exit status",
            "is that of the last command executed.",
        ],
        &[],
        &[],
    );

    let expected = concat!(
        "exit: exit [n]\n",
        "    Exit the shell.\n",
        "    \n",
        "    Exits the shell with a status of N.  If N is omitted, the exit status\n",
        "    is that of the last command executed.\n",
    );
    assert_eq!(def.help(), expected);
}

// ── Real `bash -c 'help shift'` output: no Options, an Exit Status ────────
#[test]
fn help_shift_matches_bash() {
    let def = make_def(
        "shift",
        "Shift positional parameters.",
        "shift [n]",
        &[],
        &[
            "Rename the positional parameters $N+1,$N+2 ... to $1,$2 ...  If N is",
            "not given, it is assumed to be 1.",
        ],
        &[],
        &["Returns success unless N is negative or greater than $#."],
    );

    let expected = concat!(
        "shift: shift [n]\n",
        "    Shift positional parameters.\n",
        "    \n",
        "    Rename the positional parameters $N+1,$N+2 ... to $1,$2 ...  If N is\n",
        "    not given, it is assumed to be 1.\n",
        "    \n",
        "    Exit Status:\n",
        "    Returns success unless N is negative or greater than $#.\n",
    );
    assert_eq!(def.help(), expected);
}

// ── Test against real `bash -c 'help return'` output ───────
#[test]
fn help_return_matches_bash() {
    let def = make_def(
        "return",
        "Return from a shell function.",
        "return [n]",
        &[],
        &[
            "Causes a function or sourced script to exit with the return value",
            "specified by N.  If N is omitted, the return status is that of the",
            "last command executed within the function or script.",
        ],
        &[],
        &["Returns N, or failure if the shell is not executing a function or script."],
    );

    let expected = concat!(
        "return: return [n]\n",
        "    Return from a shell function.\n",
        "    \n",
        "    Causes a function or sourced script to exit with the return value\n",
        "    specified by N.  If N is omitted, the return status is that of the\n",
        "    last command executed within the function or script.\n",
        "    \n",
        "    Exit Status:\n",
        "    Returns N, or failure if the shell is not executing a function or script.\n",
    );
    assert_eq!(def.help(), expected);
}

// ── Test multi-line flag descriptions (cd-style) ───────────
fn cd_flags() -> Vec<Flag> {
    vec![Flag::new('L').desc("force symbolic links to be followed: resolve symbolic\nlinks in DIR after processing instances of `..'").once(), Flag::new('P').desc("use the physical directory structure without following\nsymbolic links: resolve symbolic links in DIR before\nprocessing instances of `..'").once(), Flag::new('e').desc("if the -P option is supplied, and the current working\ndirectory cannot be determined successfully, exit with\na non-zero status").once()]
}

#[test]
fn help_cd_multiline_flags() {
    let def = make_def(
        "cd",
        "Change the shell working directory.",
        "cd [-L|[-P [-e]]] [-@] [dir]",
        &cd_flags(),
        &[
            "Change the current directory to DIR.  The default DIR is the value of the",
            "HOME shell variable. If DIR is \"-\", it is converted to $OLDPWD.",
            "",
            "The variable CDPATH defines the search path for the directory containing",
            "DIR.  Alternative directory names in CDPATH are separated by a colon (:).",
            "A null directory name is the same as the current directory.  If DIR begins",
            "with a slash (/), then CDPATH is not used.",
            "",
            "If the directory is not found, and the shell option `cdable_vars' is set,",
            "the word is assumed to be  a variable name.  If that variable has a value,",
            "its value is used for DIR.",
        ],
        &[
            "The default is to follow symbolic links, as if `-L' were specified.",
            "`..' is processed by removing the immediately previous pathname component",
            "back to a slash or the beginning of DIR.",
        ],
        &[
            "Returns 0 if the directory is changed, and if $PWD is set successfully when",
            "-P is used; non-zero otherwise.",
        ],
    );

    let help = def.help();
    // Verify multi-line flag continuation uses \t\t
    assert!(help.contains("  -L\tforce symbolic links to be followed: resolve symbolic\n"));
    assert!(help.contains("\t\tlinks in DIR after processing instances of `..'\n"));
    // Verify post-Options extra content
    assert!(help.contains("    The default is to follow symbolic links"));
    // Verify Exit Status
    assert!(help.contains("    Exit Status:\n    Returns 0"));
}

// ── Test extra_help (echo-style tab-formatted content) ─────
#[test]
fn help_with_extra_tab_content() {
    let def = make_def(
        "echo",
        "Write arguments to the standard output.",
        "echo [-neE] [arg ...]",
        &[],
        &[
            "Display the ARGs, separated by a single space character and followed by a",
            "newline, on the standard output.",
        ],
        &[
            "Options:",
            "  -n\tdo not append a newline",
            "  -e\tenable interpretation of the following backslash escapes",
            "  -E\texplicitly suppress interpretation of backslash escapes",
            "",
            "`echo' interprets the following backslash-escaped characters:",
            "  \\a\talert (bell)",
            "  \\b\tbackspace",
        ],
        &["Returns success unless a write error occurs."],
    );

    let help = def.help();
    // First line
    assert!(help.starts_with("echo: echo [-neE] [arg ...]\n"));
    // No auto-generated Options (no flags)
    assert!(!help.contains("    Options:\n    Options:"));
    // Extra content has manual Options
    assert!(help.contains("    Options:\n      -n\t"));
    // Backslash table
    assert!(help.contains("      \\a\talert (bell)\n"));
    // Exit Status
    assert!(help.contains("    Exit Status:\n    Returns success"));
}

// ── Test: usage() still works, short_doc only affects help ──
#[test]
fn usage_ignores_short_doc() {
    let def = make_def(
        "alias",
        "Define or display aliases.",
        "alias [-p] [name[=value] ... ]",
        &alias_flags(),
        &[],
        &[],
        &[],
    );
    assert_eq!(def.usage(), "alias [-p]");
}

#[test]
fn help_falls_back_to_usage_when_no_short_doc() {
    let def = make_def(
        "alias",
        "Define or display aliases.",
        "",
        &alias_flags(),
        &[],
        &[],
        &[],
    );
    assert!(def.help().starts_with("alias: alias [-p]\n"));
}

#[test]
fn posix_help_stays_bash_formatted() {
    let def = make_def(
        "exit",
        "Exit the shell.",
        "exit [n]",
        &[],
        &["Exits the shell with a status of N."],
        &[],
        &[],
    );
    let help = def.help();
    assert!(help.starts_with("exit: exit [n]\n"));
    assert!(!help.contains("Usage:"));
}

// Reproduces bash's `help unalias` synopsis
#[test]
fn bash_usage_repeats_a_required_rest_operand() {
    let def = Def::builder("unalias")
        .flag(Flag::new('a').desc("remove all alias definitions"))
        .rest(Positional::new("name").required())
        .build();

    assert_eq!(def.usage(), "unalias [-a] name [name ...]");
}

// Reproduces bash's `help eval` synopsis
#[test]
fn bash_usage_brackets_an_optional_rest_operand() {
    let def = Def::builder("eval").rest(Positional::new("arg")).build();

    assert_eq!(def.usage(), "eval [arg ...]");
}
