# How a scan reads argv

`Def::scan` walks the arguments once, left to right, and returns every flag in order with every
operand. Each rule below is what the reference commands do, and a test in `crates/lib/tests`
holds each one.

## Clusters and values

An argument that opens with `-` and is not `--` is a cluster of short flags, so `-abc` is `-a`,
`-b` and `-c`. A valued flag takes the rest of its cluster as the value (`-ofile`), one leading
`=` stripped (`-o=file` is `file`), or the next argument when nothing is attached (`-o file`).
A cluster ends at the first valued flag.

A valued flag refuses to take a declared option as its separated value unless the flag
`allows_hyphen_values` (the derive default), so `-o --all` reports `MissingValue` for `-o` when
`--all` exists and `UnknownFlag("--all")` when it does not.

`--` ends flag scanning, and everything after it is an operand. A bare `-` is an operand.

## GNU long options

Under `Style::Gnu`, `--name` names a long flag, `--name=value` attaches its value, and
`--name value` takes the next argument. An unambiguous prefix resolves (`--wid` for `--width`),
an ambiguous one reports `AmbiguousOption`, and `exact_long` turns prefixes off. A flag answers
to its long, its aliases and its visible aliases.

`--help` and `--version` are reserved and come back as `HelpRequested` and `VersionRequested`
unless a declared flag owns the exact name. `-h` and `-V` do the same when no flag owns the
short; `no_implicit_version` withholds `-V` and keeps `--version`.

A boolean long flag given `=value` reports `UnexpectedValue`.

## Order

Under `Style::Posix` the first operand ends flag scanning. Under `Style::Gnu` with `permute`
(the default) flags may follow operands, and `permute(false)` restores the POSIX rule for a GNU
command such as `basename`.

## Polarity

When a definition holds a `Polar` or `PolarValue` flag, an argument that opens with `+` is a
cluster too, and each flag in it records `Polarity::Off` where `-` records `On`. Without such a
flag, `+x` is an operand.

## Repeats and conflicts

Under `Style::Gnu` a flag that appeared before reports `RepeatedFlag` unless it is repeatable
(the derive default, off with `no_override` or `Flag::once`). Two flags from different
`exclusive_flags` groups report `ConflictingFlags` naming both.

## Lenient commands

A lenient definition turns an argument holding an unknown short flag into an operand instead of
an error (`echo -z`), and does the same for an unknown long option.

## Unimplemented flags

A flag declared `unimplemented` still parses, and the scan lists it under `unimplemented()`.
The derive's `parse` reports the first one as `UnimplementedFlag`; a shell that runs the
external command instead reads the list itself.

## Accepted values

A flag with `possible_values` refuses any other value with `InvalidValue`.

## Value policies

The `tag(...)` keys below change what a valued flag does when nothing is attached. Each takes
`field=default` pairs, and the default is the value the flag reports when it consumes nothing.

| Key | Without an attached value |
|---|---|
| `optional_values` | reports the default and leaves the next argument alone |
| `optional_next_values` | takes the next argument unless it reads as an option |
| `numeric_next_values` | takes the next argument when it is a numbering spec, else the default; the long form always takes the next argument |
| `optional_numeric_next_values` | takes the next argument when it is a numbering spec, else the default |
| `exact_short_defaults` | reports the default only when the short stands alone in its cluster |
| `optional_any_next_values` | takes whatever argument follows, else the default |

A numbering spec is a number, a negative number, or one ASCII character followed by digits.

`equals_only` names flags whose long form needs `=value`, `attached_values` flags whose short
form needs the value in the same cluster, `separated_values` flags whose short form needs it in
the next argument, and `prefixed_values` flags that read `-5x` as `-x 5`.

## Obsolete numeric forms

`first_numeric_value = "field"` reads a leading `-N` (`head -5`) into that flag when it is the
first argument, treats `-0` and an overflowing count as an operand, and reports
`FirstNumericValue` when `-N` appears later. `numeric_operands = "field=+"` reads `+N` and
`+N:M` (`tail +3`) into the flag, keeping the first occurrence.
