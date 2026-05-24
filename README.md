# ecmd

[![crates.io](https://img.shields.io/crates/v/ecmd.svg)](https://crates.io/crates/ecmd)
[![docs.rs](https://docs.rs/ecmd/badge.svg)](https://docs.rs/ecmd)
[![MIT](https://img.shields.io/crates/l/ecmd.svg)](LICENSE)

Type-driven argument parser for Rust. Field types determine parsing behavior.

```toml
[dependencies]
ecmd = "0.2"
```

## What it looks like

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

No attributes needed on positional fields. `bool` is a flag. `Option<T>` takes a value. `Operands` collects the rest. The struct definition is the parser.

## Type mapping

| Field type | Parses as |
|---|---|
| `bool` | `-v` sets true |
| `Polarity` | `-x` = On, `+x` = Off |
| `Option<String>` | `-o val` or `-oval` |
| `Option<T: FromStr>` | `-n 42` parsed to T |
| `Vec<PolarVal>` | `-o val` / `+o val` accumulated |
| `Operands` | everything after flags |

## Low-level API

If you need direct access to the parser:

```rust
use ecmd::parse::{scan, FlagDef, FlagKind, OnUnknown};

let flags = &[
    FlagDef { ch: 'v', kind: FlagKind::Bool, clears: &[] },
    FlagDef { ch: 'o', kind: FlagKind::Value, clears: &[] },
];

let r = scan(&["-v", "-ofile", "src"], flags, OnUnknown::Reject).unwrap();
assert_eq!(r.operands, ["src"]);
```

Handles POSIX flag bundling (`-abc`), stuck values (`-ofile`), `--` terminator, `+x` polarity, lenient mode (unknown flags become operands), and last-wins mutex groups.

## License

[MIT](LICENSE)
