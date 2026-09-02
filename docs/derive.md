# The derive

`#[derive(Command)]` reads a struct, its doc comment and its attributes, and writes `def()`,
which returns one static `Def`, and `parse()`, which scans and fills the fields. Field types
name the shape (`README.md` has the table), and the attributes below refine it.

## `#[command(...)]`

| Attribute | Meaning |
|---|---|
| `name = "cd"` | the command name, required |
| `style = "posix"` or `"gnu"` | the argument conventions, `posix` by default |
| `help_style = "bash"`, `"gnu"`, `"clap"`, `"clap_wide"` or `"util_linux"` | the help dialect when it differs from the one `style` implies |
| `lenient` | an unknown flag becomes an operand |
| `no_permute` | the first operand ends flag scanning under `gnu` |
| `no_override` | a repeated scalar flag is an error under `gnu` |
| `no_implicit_version` | `-V` is not reserved for version |
| `noop = "eE"` | short flags accepted and dropped |
| `short_doc = "cd [-LP] [dir]"` | the usage line help prints instead of the generated one |
| `extra_help("line", ...)` | lines placed after the options, replacing the doc comment's |
| `tag(key)` or `tag(key = "value")` | a key and value pair, either a policy or help key below or one for the caller |

## `#[flag(...)]`

| Attribute | Meaning |
|---|---|
| `short = 'v'` | the short character |
| `long = "verbose"` | the long name; a `gnu` command derives one from the field name when absent |
| `alias = "loud"` | another long name help does not list |
| `visible_alias = "chatty"` | another long name help lists |
| `value_name = "N"` | the value name help shows, `ARG` by default |
| `values("a", "b")` | the only values the flag accepts |
| `help_values("a", "b")` | the values help lists |
| `default = "80"` | the value an absent `Option<T>` flag takes, parsed through `FromStr` |
| `help_label = "-w, --width <N>"` | the label help prints instead of the generated one |
| `clears(other, ...)` | fields reset when this flag fires, so the last of a group wins |
| `hide` | help omits the flag |
| `unimplemented` | the flag parses but an external command implements it |
| `repeatable` | a second occurrence is accepted even under `no_override` |
| `reject_hyphen_values` | a separated value may not open with a hyphen |

A flag needs `short` or `long`. A long-only flag takes a synthetic identity above `U+E000`, so
`Parsed` never confuses it with a short.

## `#[operand(...)]`

| Attribute | Meaning |
|---|---|
| `label = "FILE"` | the label help shows instead of the field name |
| `default = "-"` | the value an absent optional positional takes, or the one operand an empty `Operands` slot holds |
| `hide` | help omits the slot |
| `required` | on `Operands`, the parse fails with `MissingRequired` when no operand reaches it |
| `spread` | help shows the slot as taking more than one value |

A positional is required when its type is `String` and optional when it is `Option<String>`, so
`required` on a positional and `default` on a required positional are compile errors, and so is
`#[operand]` on a flag field.

## The doc comment

The first paragraph of the struct's doc comment is the summary, the paragraphs after it the
description, and a field's doc comment its flag or slot description. Two markers split the rest:
the lines after `# Options` are placed after the options, and the lines after `# Exit Status`
form that section. `tag(verbatim_help)` prints the `# Options` block as the whole help page.

## Policy keys

`tag(optional_values = "field=default,other=x")` and the other keys in `docs/parsing.md` name
fields by ident or by long name.

## Help keys

| Key | Dialects | Meaning |
|---|---|---|
| `help_desc = "..."` and `version_desc = "..."` | clap, util-linux | the wording of the two reserved rows |
| `help_first` and `version_first` | clap | move the reserved rows to the top |
| `help_before = "long"` | clap | place `--help` before that option |
| `help_spaced` | clap wide, util-linux | a blank line between rows |
| `arg_row = "INDEX\tLABEL\tDESC"` | clap | a help-only argument row |
| `help_row = "INDEX\tLABEL\tDESC"` | util-linux | a help-only option row, an empty label and description for a separator |
| `help_width = "27"` and `help_pair_width = "27"` | util-linux | the description column, and the one for the reserved pair |
| `man_page = "name"` and `no_trailer` | util-linux | the name in the `(1)` trailer, or no trailer |
| `gnu_literal_tail` | gnu | the `# Options` block replaces the generated options |
| `verbatim_help` | all | the `# Options` block is the whole page |
