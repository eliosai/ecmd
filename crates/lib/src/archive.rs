//! rkyv adapters for owned command metadata

use core::fmt;

use rkyv::rancor::{Fallible, Source};
use rkyv::with::{ArchiveWith, DeserializeWith, Map, SerializeWith};
use rkyv::{Archive, Deserialize, Place, Serialize};

use crate::meta::{CommandDef, Owned, PositionalDef};
use crate::parse::FlagDef;
use crate::parse::{FlagKind, OnUnknown};
use crate::policy::{ExclusiveRule, NumericOperandRule, ValueMode, ValueRule};
use crate::style::{HelpStyle, Style};

/// A fieldless enum stored as one byte
trait Coded: Sized {
    fn code(&self) -> u8;
    fn from_code(code: u8) -> Option<Self>;
}

/// Archives a fieldless enum as its byte code
#[doc(hidden)]
#[non_exhaustive]
pub struct Code;

impl<T: Coded> ArchiveWith<T> for Code {
    type Archived = u8;
    type Resolver = ();

    fn resolve_with(field: &T, (): (), out: Place<u8>) {
        field.code().resolve((), out);
    }
}

impl<T: Coded, S: Fallible + ?Sized> SerializeWith<T, S> for Code {
    fn serialize_with(_: &T, _: &mut S) -> Result<(), S::Error> {
        Ok(())
    }
}

impl<T: Coded, D: Fallible + ?Sized> DeserializeWith<u8, T, D> for Code
where
    D::Error: Source,
{
    fn deserialize_with(field: &u8, _: &mut D) -> Result<T, D::Error> {
        T::from_code(*field).ok_or_else(|| D::Error::new(UnknownCode(*field)))
    }
}

/// A byte that names no variant of the enum it was read into
#[derive(Debug)]
struct UnknownCode(u8);

impl fmt::Display for UnknownCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown enum code {}", self.0)
    }
}

impl core::error::Error for UnknownCode {}

macro_rules! coded {
    ($ty:ty { $($variant:ident = $code:literal),+ $(,)? }) => {
        impl Coded for $ty {
            fn code(&self) -> u8 {
                match self {
                    $(Self::$variant => $code,)+
                }
            }

            fn from_code(code: u8) -> Option<Self> {
                match code {
                    $($code => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

coded!(Style { Posix = 0, Gnu = 1 });
coded!(HelpStyle { Bash = 0, Gnu = 1, Clap = 2, ClapWide = 3, UtilLinux = 4 });
coded!(OnUnknown { Reject = 0, PassThrough = 1 });
coded!(FlagKind { Bool = 0, Value = 1, Polar = 2, PolarValue = 3, Noop = 4 });
coded!(ValueMode {
    AttachedOrDefault = 0,
    NextOrDefault = 1,
    NumericNextOrDefault = 2,
    OptionalNumericNextOrDefault = 3,
    ExactShortDefault = 4,
    AnyNextOrDefault = 5,
});

/// Archive adapter for an owned flag definition
#[derive(Archive, Serialize, Deserialize)]
#[rkyv(remote = FlagDef<Owned>)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "mirrors the definition it archives"
)]
pub struct FlagDefDef {
    ch: char,
    long: String,
    aliases: Vec<String>,
    #[rkyv(with = Code)]
    kind: FlagKind,
    clears: Vec<char>,
    desc: String,
    value_name: String,
    hidden: bool,
    implemented: bool,
    repeatable: bool,
    allow_hyphen_values: bool,
    possible_values: Vec<String>,
    help_values: Vec<String>,
    default_value: String,
    help_label: String,
    visible_aliases: Vec<String>,
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
            help_values: value.help_values,
            default_value: value.default_value,
            help_label: value.help_label,
            visible_aliases: value.visible_aliases,
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
    spread: bool,
}

/// Archive adapter for an owned value rule
#[derive(Archive, Serialize, Deserialize)]
#[rkyv(remote = ValueRule<Owned>)]
pub struct ValueRuleDef {
    ch: char,
    #[rkyv(with = Code)]
    mode: ValueMode,
    default: String,
}

impl From<ValueRuleDef> for ValueRule<Owned> {
    fn from(value: ValueRuleDef) -> Self {
        Self {
            ch: value.ch,
            mode: value.mode,
            default: value.default,
        }
    }
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
            spread: value.spread,
        }
    }
}

/// Archive adapter for an owned command definition
#[derive(Archive, Serialize, Deserialize)]
#[rkyv(remote = CommandDef<Owned>)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "mirrors the definition it archives"
)]
pub struct CommandDefDef {
    name: String,
    about: String,
    short_doc: String,
    #[rkyv(with = Code)]
    style: Style,
    #[rkyv(with = Code)]
    help_style: HelpStyle,
    #[rkyv(with = Code)]
    on_unknown: OnUnknown,
    permute: bool,
    #[rkyv(with = Map<FlagDefDef>)]
    flags: Vec<FlagDef<Owned>>,
    #[rkyv(with = Map<PositionalDefDef>)]
    positionals: Vec<PositionalDef<Owned>>,
    has_rest: bool,
    rest_label: String,
    rest_hidden: bool,
    rest_desc: String,
    rest_default: String,
    rest_required: bool,
    #[rkyv(with = Map<ValueRuleDef>)]
    value_rules: Vec<ValueRule<Owned>>,
    numeric_operands: Vec<NumericOperandRule>,
    first_numeric_value: Option<char>,
    exact_long: bool,
    no_implicit_version: bool,
    equals_only: Vec<char>,
    attached_values: Vec<char>,
    separated_values: Vec<char>,
    prefixed_values: Vec<char>,
    exclusive_groups: Vec<ExclusiveRule>,
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
            rest_desc: value.rest_desc,
            rest_default: value.rest_default,
            rest_required: value.rest_required,
            value_rules: value.value_rules,
            numeric_operands: value.numeric_operands,
            first_numeric_value: value.first_numeric_value,
            exact_long: value.exact_long,
            no_implicit_version: value.no_implicit_version,
            equals_only: value.equals_only,
            attached_values: value.attached_values,
            separated_values: value.separated_values,
            prefixed_values: value.prefixed_values,
            exclusive_groups: value.exclusive_groups,
            tags: value.tags,
            description: value.description,
            extra: value.extra,
            exit_status: value.exit_status,
        }
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::indexing_slicing, reason = "tests")]
mod tests {
    use rkyv::with::With;

    use super::{ArchivedCommandDefDef, CommandDefDef};
    use crate::meta::{CommandDef, Owned};
    use crate::parse::{FlagDef, FlagKind, OnUnknown};
    use crate::policy::{ExclusiveRule, NumericOperandRule, ValueMode, ValueRule};
    use crate::style::{HelpStyle, Style};

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
                possible_values: Vec::new(),
                help_values: Vec::new(),
                default_value: String::new(),
                help_label: String::new(),
                visible_aliases: Vec::new(),
            }],
            positionals: Vec::new(),
            has_rest: true,
            rest_label: "ARGS".to_owned(),
            rest_hidden: false,
            rest_desc: "Additional values".to_owned(),
            rest_default: String::new(),
            rest_required: false,
            value_rules: vec![ValueRule {
                ch: 'f',
                mode: ValueMode::AttachedOrDefault,
                default: "local".to_owned(),
            }],
            numeric_operands: vec![NumericOperandRule {
                ch: 'f',
                prefix: '+',
            }],
            first_numeric_value: Some('f'),
            exact_long: true,
            no_implicit_version: false,
            equals_only: vec!['f'],
            attached_values: vec!['f'],
            separated_values: vec!['s'],
            prefixed_values: vec!['f'],
            exclusive_groups: vec![ExclusiveRule { ch: 'f', group: 1 }],
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
        assert_eq!(decoded.value_rules[0].default, "local");
        assert_eq!(decoded.numeric_operands[0].prefix, '+');
        assert_eq!(decoded.first_numeric_value, Some('f'));
        assert!(decoded.exact_long);
        assert_eq!(decoded.equals_only, ['f']);
        assert_eq!(decoded.attached_values, ['f']);
        assert_eq!(decoded.separated_values, ['s']);
        assert_eq!(decoded.prefixed_values, ['f']);
        assert_eq!(
            decoded.exclusive_groups.first().map(|rule| rule.group),
            Some(1)
        );
    }
}
