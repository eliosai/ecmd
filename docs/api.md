# The public API

Every public item lives at the crate root. `#[doc(hidden)] pub mod __private` carries what the
derive writes into statics and is not part of this contract. cargo-semver-checks compares each
release against the one before it, and a breaking change bumps the major.

## `Command`

The trait `#[derive(Command)]` implements. `def()` returns the static `Def`, `parse(args)` parses
one argument slice, and `parse_env()` parses the arguments of the running process.

## `Def` and `DefBuilder`

`Def::builder(name)` starts a definition. The builder takes `about`, `short_doc`, `style` (which
also picks the help dialect), `help_style`, `lenient`, `permute`, `exact_long`,
`no_implicit_version`, `flag` and `flags`, `positional` and `positionals`, `rest`, `tag` and
`tags`, `description`, `extra` and `exit_status`, and `build` returns the `Def`.

A `Def` answers `name`, `about`, `short_doc`, `style`, `help_style`, `lenient`, `permute`,
`flags`, `flag(id)`, `flag_named(name)`, `positionals`, `rest`, `tags`, `tag(key)`,
`description`, `extra` and `exit_status`. `usage()` renders the usage line, `help()` the help
page, and `scan(args)` runs the scanner and returns a `Scan` that borrows the arguments and the
definition. `with_name` and `with_tag` return the same definition renamed or tagged.

With the `rkyv` feature, `Def` archives and deserializes into an owned copy.

## `Flag` and `FlagKind`

`Flag::new(short)` names a boolean flag and `Flag::long_only(name)` one with no short, which takes
its identity from its position when a definition adds it. The setters are `long`, `alias`,
`visible_alias`, `kind`, `value`, `desc`, `hidden`, `unimplemented`, `once`,
`reject_hyphen_values`, `possible_values`, `help_values`, `default_value` and `help_label`.

The accessors are `id`, `short`, `long_name`, `aliases`, `visible_aliases`, `long_names`,
`answers_to`, `flag_kind`, `takes_value`, `description`, `value_name`, `is_hidden`,
`is_implemented`, `is_repeatable`, `allows_hyphen_values`, `accepted_values`, `listed_values`,
`default` and `label`.

`FlagKind` is `Bool`, `Value`, `Polar`, `PolarValue` or `Noop`, and the enum may grow.

## `Positional`

`Positional::new(name)` names an optional slot. The setters are `required`, `desc`, `label`,
`default_value`, `hidden` and `spread`, and the accessors `name`, `is_required`, `description`,
`shown_as`, `default`, `is_hidden` and `is_spread`.

## `Scan`, `Parsed` and `Spelling`

A `Scan<'a>` holds `flags()` as `&[Parsed<'a>]` in argument order, `operands()` as `&[&'a str]`,
and `unimplemented()` as `&[Spelling<'a>]`, the flags that appeared but are declared as
implemented elsewhere.

`Parsed` is `Bool(id)`, `Value(id, &str)`, `Polar(id, Polarity)` or `PolarValue(id, Polarity,
&str)`, and `flag()` returns the identity. `Spelling` is `Short(char)`, `Long(&str)` or
`Arg(&str)` and displays as the caller wrote it (`-x`, `--name`, `-5`). Both enums may grow.

## `Error`

`UnknownFlag`, `UnimplementedFlag`, `MissingValue`, `MissingRequired`, `InvalidValue`,
`AmbiguousOption`, `UnexpectedValue`, `RepeatedFlag`, `ConflictingFlags`, `FirstNumericValue`,
`HelpRequested` and `VersionRequested`. Every label is the flag as the caller spelled it. The
enum may grow.

## `Operands`, `Polarity` and `PolarVal`

`Operands` is the rest slot's value and dereferences to `[String]`, with `from_args`, `len`,
`is_empty`, `join`, `first`, `get` and `iter`. `Polarity` is `Unset`, `On` or `Off` with `is_on`,
`is_off` and `is_set`. `PolarVal` pairs a `polarity` with a `value`.

## `Style` and `HelpStyle`

`Style` is `Posix` or `Gnu`. `HelpStyle` is `Bash`, `Gnu`, `Clap`, `ClapWide` or `UtilLinux`, and
`from_parse_style` names the dialect a style implies. Both may grow.
