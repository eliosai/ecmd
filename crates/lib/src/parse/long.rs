//! GNU long options such as `--name` and `--name=value`

use crate::def::{Flag, FlagKind};
use crate::error::Error;
use crate::polarity::Polarity;

use super::config::ScanConfig;
use super::cursor::Cursor;
use super::state::State;
use super::value::{Form, take_value};
use super::{Parsed, Spelling};

/// Outcome of resolving a long option name against the declared flags
pub enum LongMatch<'a> {
    Flag(usize, &'a Flag),
    Help,
    Version,
    Ambiguous,
    Unknown,
}

/// One `name` or `name=value` spec whose `--` token the cursor has moved past
pub fn process_long<'a>(
    spec: &'a str,
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a>,
    state: &mut State<'a>,
) -> Result<(), Error> {
    let (name, inline) = spec
        .split_once('=')
        .map_or((spec, None), |(name, value)| (name, Some(value)));
    let spelling = Spelling::Long(name);
    match resolve_long(name, config) {
        LongMatch::Flag(position, flag) => {
            apply_long(position, flag, inline, cursor, config, state, spelling)
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
fn apply_long<'a>(
    position: usize,
    flag: &'a Flag,
    inline: Option<&'a str>,
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a>,
    state: &mut State<'a>,
    spelling: Spelling<'a>,
) -> Result<(), Error> {
    let parsed = match flag.flag_kind() {
        FlagKind::Noop => Parsed::Bool(flag.id()),
        FlagKind::Bool | FlagKind::Polar => {
            if let Some(value) = inline {
                return Err(Error::UnexpectedValue {
                    flag: spelling.to_string(),
                    value: value.to_owned(),
                });
            }
            if flag.flag_kind() == FlagKind::Bool {
                Parsed::Bool(flag.id())
            } else {
                Parsed::Polar(flag.id(), Polarity::On)
            }
        }
        FlagKind::Value | FlagKind::PolarValue => {
            let value = take_value(flag, inline, cursor, config, spelling, Form::Long)?;
            if flag.flag_kind() == FlagKind::Value {
                Parsed::Value(flag.id(), value)
            } else {
                Parsed::PolarValue(flag.id(), Polarity::On, value)
            }
        }
    };
    state.record(config, position, flag, parsed, spelling)
}

/// An exact name wins; otherwise an unambiguous prefix, unless the command demands exact names
pub fn resolve_long<'a>(name: &str, config: &ScanConfig<'a>) -> LongMatch<'a> {
    if !name.is_empty()
        && let Some((position, flag)) = config
            .flags
            .iter()
            .enumerate()
            .find(|(_, flag)| flag.answers_to(name))
    {
        return LongMatch::Flag(position, flag);
    }
    match name {
        "help" => LongMatch::Help,
        "version" => LongMatch::Version,
        _ if config.policy.exact_long => LongMatch::Unknown,
        _ => infer_long(name, config),
    }
}

/// The one declared long, or reserved action, that `name` is a prefix of
fn infer_long<'a>(name: &str, config: &ScanConfig<'a>) -> LongMatch<'a> {
    if name.is_empty() {
        return LongMatch::Unknown;
    }
    let mut found = LongMatch::Unknown;
    let mut count = 0_u32;
    for (position, flag) in config.flags.iter().enumerate() {
        if flag.long_names().any(|long| long.starts_with(name)) {
            found = LongMatch::Flag(position, flag);
            count = count.saturating_add(1);
        }
    }
    for (action, hit) in [("help", LongMatch::Help), ("version", LongMatch::Version)] {
        let shadowed = config.flags.iter().any(|flag| flag.answers_to(action));
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
