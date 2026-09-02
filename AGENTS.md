# ecmd

ecmd parses POSIX and GNU argv the way the reference commands do. `crates/lib` is the `ecmd`
crate and `crates/derive` is the `ecmd-derive` proc macro it re-exports. Nothing else is published.

## Layout

- `crates/lib/src` holds the scanner, the command definition and the help renderers, and
  `crates/lib/tests` holds the integration tests, which reach only the public API
- `crates/derive/src` reads the `command`, `flag` and `operand` attributes and emits one
  `Command` impl over a static definition
- `docs/` explains what the code cannot: `api.md` the public surface, `parsing.md` the scan
  rules, `derive.md` the attributes, `releasing.md` the pipeline, `todo.md` the open work
- `scripts/` holds the checks `just` runs, and `.github/workflows` runs the same recipes

## Commands

Every task has a `just` recipe, and the gate runs nothing a recipe does not run. `just check`
scans comments, docs and layout, format-checks, type-checks every feature set and runs clippy
with warnings denied. `just test` runs nextest, `just test-doc` the doc examples, `just doc-check`
the docs.rs build, `just package-check` the crates.io packages, `just semver-check` the public API
against the last release, `just msrv` the 1.88 build, `just audit` cargo-deny and `just ci` all of
it. `just hooks` installs the prek hooks, so a commit runs `just check` and a push runs `just test`.

## Comments And Prose

ONE LINE. Every comment and every doc comment is exactly one line. No second line, no blank `///`,
no paragraph, no `# Errors` or `# Panics` or `# Safety` section, no example block. This is absolute
and applies to modules, types, fields, functions, macros, tests, and inline comments alike. The one
exception is a fixture under `crates/lib/tests`, whose doc comment is the help text under test.

The line states what the item is. It does not explain why the item exists, who calls it, what it
returns on failure, or anything the signature already says. Do not end it with a period.

If one line cannot carry the meaning, the name is wrong or the item does too much. Fix the code, do
not add a second line.

The crate doc is the README, pulled in with `include_str!`, so every README example is a doc test.
Project prose uses active voice, concrete terms, and short paragraphs. Read `stop-slop` and
`josh-voice` before writing it. Do not add speculative architecture or duplicate an existing document.

## Visibility

Never write `pub(crate)`, `pub(super)`, `pub(in path)`, or any other restricted visibility. An item
is a plain private `fn` unless another module needs it, and then it is `pub`. Keep the private form
whenever it still compiles.

The public surface is the crate root. Every module is private and the root re-exports what a caller
may name; the derive's plumbing lives under `#[doc(hidden)] pub mod __private` and is not API.
Every public enum is `#[non_exhaustive]`, every public struct keeps its fields private, and
`docs/api.md` lists every public item.

## Code Quality

- keep functions and methods at 25 lines or less and files under 400 lines; split by behavior first
- never pair `x.rs` with an `x/` directory, and never use `#[path]`; a module with children is `x/mod.rs`
- production code returns `Error`; `unwrap`, `expect` and `panic` live only in tests
- the scanner allocates nothing per flag; a label is built only inside the `Err` it names
- read `rust-best-practices`, `coding-guidelines` and `stop-slop` before every Rust change, and
  `codebase-design` before changing a module boundary or a public signature

## Tests

- integration tests live in `crates/lib/tests` and exercise the public API alone; a unit test lives
  in `#[cfg(test)] mod tests` inside the file that owns the logic it exercises
- run tests with `cargo nextest`, never `cargo test`; doc examples run through `just test-doc`
- every test proves one exact behavior, and a help test compares against the reference command's
  real output
- read `tdd` and `rust-testing` before changing behavior or tests

## Releases

Every merge to `main` may release. The release workflow reads the commits since the last tag:
a breaking API change reported by cargo-semver-checks bumps the major, a `feat` the minor, a
`fix`, `perf` or `refactor` the patch, and anything else ships nothing. A breaking pull request
carries the `semver-major` label or the semver gate fails it. `docs/releasing.md` has the rest.

## Skills

Read the matching skill in `.agents/skills` before touching its domain. `.claude/skills` points to
the same directory.

- `stop-slop` and `josh-voice` for prose, `tdd` and `rust-testing` for tests
- `rust-best-practices`, `coding-guidelines` and `codebase-design` for Rust
- `code-review` and `thermo-nuclear-code-quality-review` for every review
- `gh-stack` for pull requests, `grilling` when asked to stress-test a decision, `prek` for hooks

Do not edit a copied skill as part of product work. Update skills in their own change.

## Commits

Use conventional commits. Write the subject line and stop. Keep the subject under 60 characters,
in the imperative, with no trailing period. Never add a trailer. No `Co-Authored-By`, no
`Generated-with`, no attribution of any kind. One commit does one thing, and it compiles and passes
its tests on its own.

A pull request adds at most 1000 lines, or 3000 with the `mechanical` label for verbatim moves,
renames and deletes. Only a reviewer adds `size-exempt`.

## Enforcement

`just check` runs `scripts/comment-scan.sh`, which fails on any comment longer than one line,
`scripts/doc-scan.sh`, which fails on any Rust example the doc tests skip, and
`scripts/layout-scan.sh`, which fails on scoped visibility or a module file paired with a directory.
Fix a violation by deleting lines or renaming, never by rewording around the rule.
