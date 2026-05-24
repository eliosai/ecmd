# ecmd

Argument parser for Rust. Type-driven, zero dependencies, strict.

Field types determine parsing behavior. `bool` is a flag, `Option<String>` takes a value, `Operands` collects the rest. A proc macro (WIP) generates the parser from your struct definition.

## Example

```rust
use ecmd::meta::Command;
use ecmd::parse::{scan, FlagDef, FlagKind, OnUnknown};
use ecmd::operands::Operands;

// What the derive will generate:
let flags = &[
    FlagDef { ch: 'v', kind: FlagKind::Bool, clears: &[] },
    FlagDef { ch: 'o', kind: FlagKind::Value, clears: &[] },
];

let result = scan(&["-v", "-o", "out.txt", "input.rs"], flags, OnUnknown::Reject).unwrap();
// result.flags = [Bool('v'), Value('o', "out.txt")]
// result.operands = ["input.rs"]
```

With the derive macro (coming soon):

```rust
#[derive(Command)]
#[command(name = "grep", style = "posix")]
struct Grep {
    #[flag(short = 'i')]
    ignore_case: bool,
    #[flag(short = 'n')]
    line_numbers: bool,
    #[flag(short = 'c')]
    count: bool,
    pattern: String,
    files: Operands,
}
```

## Features

- **Zero runtime dependencies.** stdlib only.
- **Type-driven.** Field type determines parse behavior. No stringly-typed config.
- **Shell-native.** Supports `+x`/`-x` polarity flags (`set +e`, `declare -i`).
- **Lenient mode.** Echo-style parsing where unknown flags become operands.
- **POSIX-correct.** Flag bundling, stuck values, `--` terminator, bare `-` as operand.
- **Small.** ~700 lines of implementation. Compiles in under a second.

## Types

| Field type | What it parses |
|---|---|
| `bool` | `-v` sets true |
| `Polarity` | `-x` = On, `+x` = Off |
| `Option<String>` | `-o val` or `-oval` |
| `Option<T: FromStr>` | `-n 42` parsed |
| `Vec<PolarVal>` | `-o val` / `+o val` accumulated |
| `Operands` | everything after flags |

## Status

The core parser is done and tested (54 tests, zero clippy warnings). The derive macro is the next step.

```
rust/
  core/     # ecmd crate (parser, types, trait)
  derive/   # proc macro (planned)
  tests/    # integration tests
```

## License

MIT
