# ecmd

[![crates.io](https://img.shields.io/crates/v/ecmd.svg)](https://crates.io/crates/ecmd)
[![docs.rs](https://docs.rs/ecmd/badge.svg)](https://docs.rs/ecmd)
[![MIT](https://img.shields.io/crates/l/ecmd.svg)](https://github.com/eliosai/ecmd/blob/main/LICENSE)

ecmd parses argv the way POSIX and GNU commands do, from a struct whose field types name the
shape. `bool` is a flag, `Option<T>` takes a value, `Operands` collects the rest, and the derive
writes the parser together with a static definition that renders help in the dialect of the
command it copies (bash builtins, GNU coreutils, clap and util-linux).

```toml
[dependencies]
ecmd = "1"
```

## A command from a struct

```rust
use ecmd::{Command, Operands};

/// Search for PATTERN in each FILE.
#[derive(Command)]
#[command(name = "grep", style = "posix")]
struct Grep {
    /// ignore case distinctions
    #[flag(short = 'i')]
    ignore_case: bool,
    /// print line numbers
    #[flag(short = 'n')]
    line_numbers: bool,
    /// stop after NUM matches
    #[flag(short = 'm', value_name = "NUM")]
    max_count: Option<u32>,
    pattern: String,
    files: Operands,
}

let grep = Grep::parse(&["-in", "-m", "3", "hello", "a.txt", "b.txt"]).unwrap();
assert!(grep.ignore_case && grep.line_numbers);
assert_eq!(grep.max_count, Some(3));
assert_eq!(grep.pattern, "hello");
assert_eq!(&*grep.files, &["a.txt", "b.txt"]);
```

`Grep::parse_env()` parses the arguments of the running process. The struct needs no attribute on
a positional field, and a flag needs only its `short` or its `long`. `docs/derive.md` lists every
attribute.

| Field type | Parses as |
|---|---|
| `bool` | `-v` sets it |
| `Polarity` | `-x` is `On`, `+x` is `Off` |
| `Option<String>` | `-o val`, `-oval` or `--out=val` |
| `Option<T: FromStr>` | the same, parsed into `T` |
| `Vec<String>` | `-o a -o b` accumulated |
| `Vec<PolarVal>` | `-o val` and `+o val` accumulated with their sign |
| `String` | a required positional |
| `Option<String>` without `#[flag]` | an optional positional |
| `Operands` | every operand past the positionals |

## Help in the dialect of the original

The doc comment on the struct is the help text, and `style` picks the dialect: a POSIX command
prints help the way bash prints `help cd`, and a GNU command prints a `Usage:` line with
`-c, --long` rows the way coreutils do. `help_style = "clap"`, `"clap_wide"` and `"util_linux"`
pick the other dialects.

```rust
use ecmd::Command;

/// Change the shell working directory.
///
/// Change the current directory to DIR.
///
/// # Exit Status
/// Returns 0 if the directory is changed.
#[derive(Command)]
#[command(name = "cd", short_doc = "cd [-LP] [dir]")]
struct Cd {
    /// force symbolic links to be followed
    #[flag(short = 'L')]
    logical: bool,
    /// use the physical directory structure
    #[flag(short = 'P')]
    physical: bool,
    dir: Option<String>,
}

let help = Cd::def().help();
assert!(help.starts_with("cd: cd [-LP] [dir]\n    Change the shell working directory.\n"));
assert!(help.contains("    Options:\n      -L\tforce symbolic links to be followed\n"));
assert!(help.ends_with("    Exit Status:\n    Returns 0 if the directory is changed.\n"));
```

## A definition at runtime

`Def::builder` builds the same definition without a struct, for a command whose shape arrives at
runtime, and `Def::scan` runs the scanner alone. A scan borrows argv, so it allocates nothing per
flag and hands back `&str` values.

```rust
use ecmd::{Def, Flag, Parsed, Positional, Style};

let ls = Def::builder("ls")
    .about("List directory contents.")
    .style(Style::Gnu)
    .flag(Flag::new('a').long("all").desc("do not ignore entries starting with ."))
    .flag(Flag::new('w').long("width").value("COLS").desc("assume screen width"))
    .rest(Positional::new("files").spread())
    .build();

let scan = ls.scan(&["--all", "-w80", "src", "docs"]).unwrap();
assert_eq!(scan.flags(), [Parsed::Bool('a'), Parsed::Value('w', "80")]);
assert_eq!(scan.operands(), ["src", "docs"]);
assert!(ls.help().starts_with("Usage: ls [-a] [-w COLS] [args...]\n"));
```

The scanner handles clusters (`-abc`), attached values (`-ofile`, `-o=file`), separated values
(`-o file`), `--long=value` and `--long value`, unambiguous long prefixes (`--wid` for `--width`),
the `--` terminator, `+x` polarity, GNU permutation (flags after operands), lenient mode (an
unknown flag becomes an operand), repeat refusal under GNU conventions, exclusive groups, and the
obsolete numeric forms of `head`, `tail` and `pr`. `docs/parsing.md` states each rule.

## Errors

Every refusal is one `Error` variant carrying the flag as the caller spelled it, and `--help`
and `--version` come back as `HelpRequested` and `VersionRequested` so the caller prints and
exits.

```rust
use ecmd::{Def, Error, Flag, Style};

let def = Def::builder("nl")
    .style(Style::Gnu)
    .flag(Flag::new('n').long("number").value("N"))
    .build();

assert_eq!(def.scan(&["--number"]).unwrap_err(), Error::MissingValue("--number".to_owned()));
assert_eq!(def.scan(&["--bogus"]).unwrap_err(), Error::UnknownFlag("--bogus".to_owned()));
assert_eq!(def.scan(&["--help"]).unwrap_err(), Error::HelpRequested);
```

## Features

`derive` (on by default) re-exports `#[derive(Command)]` from `ecmd-derive`. `rkyv` derives
`rkyv::Archive`, `Serialize` and `Deserialize` for `Def`, so a definition built at runtime can
travel inside serialized state and come back as an owned `Def`.

## Layout

- `crates/lib` is the `ecmd` crate: `def/` holds `Def`, `Flag` and `Positional` with their
  builders, `parse/` the scanner, `help/` one file per dialect
- `crates/derive` is `ecmd-derive`, the proc macro `ecmd` re-exports
- `docs/api.md` lists every public item, `docs/parsing.md` the scan rules, `docs/derive.md` the
  attributes, `docs/releasing.md` the pipeline, `docs/todo.md` the open work

## Building and testing

```text
just check          # comment, doc and layout scans, fmt, check, clippy -D warnings
just test           # the suite under cargo nextest
just test-doc       # every example in this README and the docs
just doc-check      # the docs.rs build with warnings denied
just semver-check   # the public API against the last release
just ci             # all of it, the way the gate runs
```

## License

MIT
