//! GNU long options such as `--name` and `--name=value`

use crate::error::Error;
use crate::meta::Storage;
use crate::polarity::Polarity;

use super::config::ScanConfig;
use super::cursor::Cursor;
use super::state::State;
use super::value::{Form, take_value};
use super::{FlagDef, FlagKind, Parsed, Spelling};

/// Outcome of resolving a long option name against the declared flags
pub enum LongMatch<'a, S: Storage> {
    Flag(usize, &'a FlagDef<S>),
    Help,
    Version,
    Ambiguous,
    Unknown,
}

/// One `name` or `name=value` spec whose `--` token the cursor has moved past
pub fn process_long<'a, S: Storage, P: Storage>(
    spec: &'a str,
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a, S, P>,
    state: &mut State<'a>,
) -> Result<(), Error> {
    let (name, inline) = spec
        .split_once('=')
        .map_or((spec, None), |(name, value)| (name, Some(value)));
    let spelling = Spelling::Long(name);
    match resolve_long(name, config) {
        LongMatch::Flag(position, def) => {
            apply_long(position, def, inline, cursor, config, state, spelling)
        }
        LongMatch::Help | LongMatch::Version if inline.is_some() => Err(Error::UnexpectedValue {
            flag: spelling.to_string(),
            value: inline.unwrap_or_default().to_owned(),
        }),
        LongMatch::Help => Err(Error::HelpRequested),
        LongMatch::Version => Err(Error::VersionRequested),
        LongMatch::Ambiguous => Err(Error::AmbiguousOption(spelling.to_string())),
        LongMatch::Unknown => Err(Error::UnknownFlag(spelling.to_string())),
    }
}

/// Record a resolved long flag, pulling a value from `=inline` or the next argument
fn apply_long<'a, S: Storage, P: Storage>(
    position: usize,
    def: &'a FlagDef<S>,
    inline: Option<&'a str>,
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a, S, P>,
    state: &mut State<'a>,
    spelling: Spelling<'a>,
) -> Result<(), Error> {
    let parsed = match def.kind {
        FlagKind::Noop => Parsed::Bool(def.ch),
        FlagKind::Bool | FlagKind::Polar => {
            if let Some(value) = inline {
                return Err(Error::UnexpectedValue {
                    flag: spelling.to_string(),
                    value: value.to_owned(),
                });
            }
            if def.kind == FlagKind::Bool {
                Parsed::Bool(def.ch)
            } else {
                Parsed::Polar(def.ch, Polarity::On)
            }
        }
        FlagKind::Value | FlagKind::PolarValue => {
            let value = take_value(def, inline, cursor, config, spelling, Form::Long)?;
            if def.kind == FlagKind::Value {
                Parsed::Value(def.ch, value)
            } else {
                Parsed::PolarValue(def.ch, Polarity::On, value)
            }
        }
    };
    state.record(config, position, def, parsed, spelling)
}

/// An exact name wins; otherwise an unambiguous prefix, unless the command demands exact names
pub fn resolve_long<'a, S: Storage, P: Storage>(
    name: &str,
    config: &ScanConfig<'a, S, P>,
) -> LongMatch<'a, S> {
    if !name.is_empty()
        && let Some((position, def)) = config
            .flags
            .iter()
            .enumerate()
            .find(|(_, flag)| long_names(flag).any(|long| long == name))
    {
        return LongMatch::Flag(position, def);
    }
    match name {
        "help" => LongMatch::Help,
        "version" => LongMatch::Version,
        _ if config.policy.exact_long => LongMatch::Unknown,
        _ => infer_long(name, config),
    }
}

/// Every long name a flag answers to: its long, its aliases, and its visible aliases
fn long_names<S: Storage>(flag: &FlagDef<S>) -> impl Iterator<Item = &str> {
    core::iter::once(flag.long.as_ref())
        .chain(flag.aliases.as_ref().iter().map(AsRef::as_ref))
        .chain(
            flag.visible_aliases
                .as_ref()
                .iter()
                .map(AsRef::as_ref)
                .filter(|alias| alias.chars().count() > 1),
        )
        .filter(|name| !name.is_empty())
}

/// The one declared long, or reserved action, that `name` is a prefix of
fn infer_long<'a, S: Storage, P: Storage>(
    name: &str,
    config: &ScanConfig<'a, S, P>,
) -> LongMatch<'a, S> {
    if name.is_empty() {
        return LongMatch::Unknown;
    }
    let mut found = LongMatch::Unknown;
    let mut count = 0_u32;
    for (position, def) in config.flags.iter().enumerate() {
        if long_names(def).any(|long| long.starts_with(name)) {
            found = LongMatch::Flag(position, def);
            count = count.saturating_add(1);
        }
    }
    for (action, hit) in [("help", LongMatch::Help), ("version", LongMatch::Version)] {
        let shadowed = config
            .flags
            .iter()
            .any(|flag| long_names(flag).any(|long| long == action));
        if action.starts_with(name) && !shadowed {
            found = hit;
            count = count.saturating_add(1);
        }
    }
    match count {
        1 => found,
        0 => LongMatch::Unknown,
        _ => LongMatch::Ambiguous,
    }
}
