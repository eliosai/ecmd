# Changelog

## 1.0.0 (2026-09-02)


### Features
- Core parser with types, POSIX flag parsing, and Command trait
- Derive macro for type-driven argument parsing
- Doc comments, value_name, usage/help generation
- Generic tags on CommandDef
- Add prelude module for single-import ergonomics
- Bash-compatible help sections with doc comment markers
- GNU long-option support for 0.5.3
- No_permute + long-only options for 0.5.4
- Aliases, hidden flags, short = separator for 0.5.5
- Complete command metadata and GNU parsing
- Add command-specific GNU parser policies
- Refine GNU flag value policies
- ecmd: Preserve exact long-option parsing
- ecmd: Parse leading numeric option values
- ecmd: Parse legacy numeric operands
- ecmd: Require separated short values
- parser: Support position-sensitive value defaults
- parser: Let a command opt out of the implicit -V reservation
- parser: Report every missing required positional in one error
- meta: Render help in a declared dialect, adding the clap layout
- meta: Describe operand labels, defaults, and hiding for generated help
- meta: Keep the line breaks an author wrote in a command summary
- parser: Declare a flag's accepted values and default, and show them in help
- parser: Describe operand spread and aliases that help lists
- parser: Separate the values help lists from the values a flag accepts
- meta: Name a short flag's value, keep authored indentation, place the help entry
- parser: Treat a one-character visible alias as a short flag only
- meta: Render the next-line help layout and list only the values help declares
- core: Give FlagDef and CommandDef a zero-valued base
- meta: Render the util-linux help dialect
- meta: Let a util-linux command declare a literal help row
- meta: Render extra blocks and separator rows in util-linux help
- meta: Let a GNU command author its own options tail
- meta: Let a bespoke command author its whole help page
- meta: Let a clap command author an extra argument row
- parser: Restore command policy errors
- parser: Apply typed command policies
- parser: Complete legacy option policies
- def: One definition type behind a root API
- derive: Honor required and default at parse time


### Fixes
- Ci ubuntu-only, add readme for crates.io, bump 0.2.1
- Workspace dependency for ecmd-derive, remove stale lints
- Skip empty-desc flags in Options, no separator before extra after Options
- Revert extra separator removal, keep empty-desc filter
- Reject repeated GNU noop flags
- ecmd: Preserve polar short values
- ecmd: Support long-only optional values
- parser: Constrain obsolete numeric values
- parser: Preserve tagged rules in owned metadata
- parser: Preserve rejected option values
- meta: Keep a real -h flag out of the util-linux help pair
- parser: Enforce strict long options
- parser: Preserve clustered value boundaries
- parser: Bound obsolete numeric options


### Refactoring
- parse: Split the scanner into a module tree
- parse: One value path over a borrowed scan
- help: Split metadata from the help dialects
- derive: Split codegen and attrs into module trees


### Documentation
- Clean up readme, remove status section
- Rewrite readme for crates.io
- Write the README as the crate doc and the reference docs


### Housekeeping
- Add CI and automated publish workflows
- Bump version to 0.5.0
- meta: Pin the util-linux dialect at the renderer
- workspace: Flatten repository layout
- workspace: Move to lib and derive crates, lint clean
- Add just, prek, scans, workflows and agent skills
- Release from main with semver-driven versions
- Package the README and LICENSE inside each crate
- Let the release bump reach the index


### Merge
- Integrate exact option parsing
- parser: Reconcile help metadata and typed policies


### Style
- ecmd: Format parser extensions

