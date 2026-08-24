//! rkyv adapters for owned command metadata

use rkyv::with::Map;
use rkyv::{Archive, Deserialize, Serialize};

use crate::meta::{CommandDef, Owned, PositionalDef};
use crate::parse::FlagDef;
use crate::parse::{FlagKind, OnUnknown};
use crate::style::{HelpStyle, Style};

/// Archive adapter for an owned flag definition
#[derive(Archive, Serialize, Deserialize)]
#[rkyv(remote = FlagDef<Owned>)]
pub struct FlagDefDef {
    ch: char,
    long: String,
    aliases: Vec<String>,
    kind: FlagKind,
    clears: Vec<char>,
    desc: String,
    value_name: String,
    hidden: bool,
    implemented: bool,
    repeatable: bool,
    allow_hyphen_values: bool,
    possible_values: Vec<String>,
    default_value: String,
}

impl From<FlagDefDef> for FlagDef<Owned> {
    fn from(value: FlagDefDef) -> Self {
        Self {
            ch: value.ch,
            long: value.long,
            aliases: value.aliases,
            kind: value.kind,
            clears: value.clears,
            desc: value.desc,
            value_name: value.value_name,
            hidden: value.hidden,
            implemented: value.implemented,
            repeatable: value.repeatable,
            allow_hyphen_values: value.allow_hyphen_values,
            possible_values: value.possible_values,
            default_value: value.default_value,
        }
    }
}

/// Archive adapter for an owned positional definition
#[derive(Archive, Serialize, Deserialize)]
#[rkyv(remote = PositionalDef<Owned>)]
pub struct PositionalDefDef {
    name: String,
    required: bool,
    desc: String,
    label: String,
    default_value: String,
    hidden: bool,
}

impl From<PositionalDefDef> for PositionalDef<Owned> {
    fn from(value: PositionalDefDef) -> Self {
        Self {
            name: value.name,
            required: value.required,
            desc: value.desc,
            label: value.label,
            default_value: value.default_value,
            hidden: value.hidden,
        }
    }
}

/// Archive adapter for an owned command definition
#[derive(Archive, Serialize, Deserialize)]
#[rkyv(remote = CommandDef<Owned>)]
pub struct CommandDefDef {
    name: String,
    about: String,
    short_doc: String,
    style: Style,
    help_style: HelpStyle,
    on_unknown: OnUnknown,
    permute: bool,
    #[rkyv(with = Map<FlagDefDef>)]
    flags: Vec<FlagDef<Owned>>,
    #[rkyv(with = Map<PositionalDefDef>)]
    positionals: Vec<PositionalDef<Owned>>,
    has_rest: bool,
    rest_label: String,
    rest_hidden: bool,
    tags: Vec<(String, String)>,
    description: Vec<String>,
    extra: Vec<String>,
    exit_status: Vec<String>,
}

impl From<CommandDefDef> for CommandDef<Owned> {
    fn from(value: CommandDefDef) -> Self {
        Self {
            name: value.name,
            about: value.about,
            short_doc: value.short_doc,
            style: value.style,
            help_style: value.help_style,
            on_unknown: value.on_unknown,
            permute: value.permute,
            flags: value.flags,
            positionals: value.positionals,
            has_rest: value.has_rest,
            rest_label: value.rest_label,
            rest_hidden: value.rest_hidden,
            tags: value.tags,
            description: value.description,
            extra: value.extra,
            exit_status: value.exit_status,
        }
    }
}

#[cfg(test)]
mod tests {
    use rkyv::with::With;

    use super::{ArchivedCommandDefDef, CommandDefDef};
    use crate::meta::{CommandDef, Owned};
    use crate::parse::{FlagDef, FlagKind, OnUnknown};
    use crate::style::Style;

    #[test]
    fn owned_command_definition_round_trips() {
        let definition = CommandDef::<Owned> {
            name: "greet".to_owned(),
            about: "Print a greeting".to_owned(),
            short_doc: String::new(),
            style: Style::Posix,
            help_style: HelpStyle::Bash,
            on_unknown: OnUnknown::Reject,
            permute: false,
            flags: vec![FlagDef {
                ch: 'f',
                long: "ftp-port".to_owned(),
                aliases: Vec::new(),
                kind: FlagKind::Value,
                clears: Vec::new(),
                desc: String::new(),
                value_name: "ADDRESS".to_owned(),
                hidden: false,
                implemented: false,
                repeatable: false,
                allow_hyphen_values: true,
                possible_values: &[],
                default_value: "",
            }],
            positionals: Vec::new(),
            has_rest: true,
            tags: vec![("kind".to_owned(), "extension".to_owned())],
            description: Vec::new(),
            extra: Vec::new(),
            exit_status: Vec::new(),
        };

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(
            With::<CommandDef<Owned>, CommandDefDef>::cast(&definition),
        )
        .expect("serialize definition");
        let archived = rkyv::access::<ArchivedCommandDefDef, rkyv::rancor::Error>(&bytes)
            .expect("access definition");
        let decoded = rkyv::deserialize::<CommandDef<Owned>, rkyv::rancor::Error>(With::<
            ArchivedCommandDefDef,
            CommandDefDef,
        >::cast(
            archived
        ))
        .expect("deserialize definition");

        assert_eq!(decoded.name(), definition.name());
        assert_eq!(decoded.usage(), definition.usage());
        assert_eq!(decoded.help(), definition.help());
        assert!(!decoded.flags()[0].implemented);
        assert!(!decoded.flags()[0].repeatable);
        assert!(decoded.flags()[0].allow_hyphen_values);
    }
}
