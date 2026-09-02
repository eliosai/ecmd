# Open work

Only unfinished work belongs here. Git history carries completed plans and measurements.

## 1.0

- Replace `CommandDef<S: Storage>` with one `Def` over `Cow<'static>`, delete `archive.rs` and `into_owned`
- Split `codegen.rs`, honor `#[operand(required)]`, `default =`, `values(...)` and `visible_alias` at parse time
- Collapse the root API to the trait, `Def`, `Flag`, `Scan`, `Parsed`, `Error`, `Operands`, `Polarity`, `PolarVal`, `Style`, `HelpStyle`
- Write the README as the crate doc and `docs/api.md`, `parsing.md`, `derive.md`, `releasing.md`
- Land the release workflow, `cliff.toml`, the deploy key and the `main` ruleset, then ship 1.0.0

## 1.x

- `--no-` negation for GNU boolean flags
- Environment fallback for a valued flag
- Required flags
- Subcommands
- `OsStr` argv
- Positionals in the bash, GNU and util-linux help dialects
