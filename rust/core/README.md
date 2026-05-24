# ecmd

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

No attributes needed on positional fields. `bool` is a flag. `Option<T>` takes a value. `Operands` collects the rest.

## Type mapping

| Field type | Parses as |
|---|---|
| `bool` | `-v` sets true |
| `Polarity` | `-x` = On, `+x` = Off |
| `Option<String>` | `-o val` or `-oval` |
| `Option<T: FromStr>` | `-n 42` parsed to T |
| `Vec<PolarVal>` | `-o val` / `+o val` accumulated |
| `Operands` | everything after flags |

Handles POSIX flag bundling (`-abc`), stuck values (`-ofile`), `--` terminator, `+x` polarity, lenient mode, and last-wins mutex groups.

## License

[MIT](https://github.com/eliosai/ecmd/blob/main/rust/core/LICENSE)
