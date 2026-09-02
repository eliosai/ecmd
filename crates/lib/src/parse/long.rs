//! GNU long options such as `--name` and `--name=value`

use crate::error::Error;
use crate::meta::Storage;
use crate::polarity::Polarity;
use crate::policy::ValueMode;
use crate::style::Style;

use super::config::ScanConfig;
use super::cursor::Cursor;
use super::reject::{reject_latest_conflict, reject_repeat};
use super::value::{is_numbering_spec, is_option_boundary, reject_option_value};
use super::{FlagDef, FlagKind, Parsed, ScanResult};

/// Outcome of resolving a long-option name against the declared flags.
pub enum LongMatch<'a, S: Storage> {
    Flag(&'a FlagDef<S>),
    Help,
    Version,
    Ambiguous,
    Unknown,
}

/// Parse one `--name` / `--name=value` token. Cursor is on that token.
pub fn process_long<S: Storage, P: Storage>(
    spec: &str,
    cursor: &mut Cursor<'_>,
    config: &ScanConfig<'_, S, P>,
    result: &mut ScanResult,
) -> Result<(), Error> {
    let (name, inline) = spec
        .split_once('=')
        .map_or((spec, None), |(n, v)| (n, Some(v)));
    cursor.advance();
    match resolve_long(name, config.flags, config.policy.exact_long) {
        LongMatch::Flag(def) => {
            apply_long(def, name, inline, cursor, config, result)?;
            reject_latest_conflict(result, config, &format!("--{name}"))
        }
        LongMatch::Help | LongMatch::Version if inline.is_some() => Err(Error::UnexpectedValue {
            flag: format!("--{name}"),
            value: inline.unwrap_or_default().to_owned(),
        }),
        LongMatch::Help => Err(Error::HelpRequested),
        LongMatch::Version => Err(Error::VersionRequested),
        LongMatch::Ambiguous => Err(Error::AmbiguousOption(format!("--{name}"))),
        LongMatch::Unknown => Err(Error::UnknownFlag(format!("--{name}"))),
    }
}

/// Record a resolved long flag, pulling a value from `=inline` or the next arg.
pub fn apply_long<S: Storage, P: Storage>(
    def: &FlagDef<S>,
    name: &str,
    inline: Option<&str>,
    cursor: &mut Cursor<'_>,
    config: &ScanConfig<'_, S, P>,
    result: &mut ScanResult,
) -> Result<(), Error> {
    if !def.implemented {
        result.unimplemented.push(format!("--{name}"));
    }
    match def.kind {
        FlagKind::Noop => {
            reject_repeat(def, result, &format!("--{name}"), Style::Gnu)?;
            result.flags.push(Parsed::Bool(def.ch));
            Ok(())
        }
        FlagKind::Bool | FlagKind::Polar => apply_long_flag(def, name, inline, result),
        FlagKind::Value | FlagKind::PolarValue => {
            apply_long_value(def, name, inline, cursor, config, result)
        }
    }
}

/// A long bool/polar flag: presence sets it; an inline `=value` is an error.
pub fn apply_long_flag<S: Storage>(
    def: &FlagDef<S>,
    name: &str,
    inline: Option<&str>,
    result: &mut ScanResult,
) -> Result<(), Error> {
    if let Some(value) = inline {
        return Err(Error::UnexpectedValue {
            flag: format!("--{name}"),
            value: value.to_owned(),
        });
    }
    reject_repeat(def, result, &format!("--{name}"), Style::Gnu)?;
    result.flags.push(if matches!(def.kind, FlagKind::Bool) {
        Parsed::Bool(def.ch)
    } else {
        Parsed::Polar(def.ch, Polarity::On)
    });
    Ok(())
}

/// A long value flag: value comes from `=inline` or the next token.
pub fn apply_long_value<S: Storage, P: Storage>(
    def: &FlagDef<S>,
    name: &str,
    inline: Option<&str>,
    cursor: &mut Cursor<'_>,
    config: &ScanConfig<'_, S, P>,
    result: &mut ScanResult,
) -> Result<(), Error> {
    // GNU getopt_long parity: a value flag consumes the next token even when it looks like an option.
    let label = format!("--{name}");
    if inline.is_none() && config.policy.requires_equals(def.ch) {
        return Err(Error::UnknownFlag(label));
    }
    let value = match (
        inline,
        config
            .policy
            .value(def.ch)
            .map(|rule| (rule.mode, rule.default.as_ref())),
    ) {
        (Some(value), _) => value.to_owned(),
        (None, Some((ValueMode::AttachedOrDefault, default))) => default.to_owned(),
        (None, Some((ValueMode::NextOrDefault, default))) => {
            take_long_next_or_default(cursor, config.flags, default, &label)?
        }
        (None, Some((ValueMode::NumericNextOrDefault, default))) => {
            let _ = default;
            cursor.take_next(&label)?
        }
        (None, Some((ValueMode::OptionalNumericNextOrDefault, default))) => {
            take_long_optional_numeric_or_default(cursor, default, &label)?
        }
        (None, Some((ValueMode::AnyNextOrDefault, default))) => {
            take_long_any_or_default(cursor, default, &label)?
        }
        (None, _) => {
            reject_option_value(
                cursor.peek(),
                def,
                config.flags,
                config.policy.exact_long,
                &label,
            )?;
            cursor.take_next(&label)?
        }
    };
    reject_repeat(def, result, &format!("--{name}"), Style::Gnu)?;
    result.flags.push(if matches!(def.kind, FlagKind::Value) {
        Parsed::Value(def.ch, value)
    } else {
        Parsed::PolarValue(def.ch, Polarity::On, value)
    });
    Ok(())
}

pub fn take_long_next_or_default<S: Storage>(
    cursor: &mut Cursor<'_>,
    flags: &[FlagDef<S>],
    default: &str,
    label: &str,
) -> Result<String, Error> {
    if cursor
        .peek()
        .is_some_and(|value| !is_option_boundary(value, flags))
    {
        cursor.take_next(label)
    } else {
        Ok(default.to_owned())
    }
}

pub fn take_long_optional_numeric_or_default(
    cursor: &mut Cursor<'_>,
    default: &str,
    label: &str,
) -> Result<String, Error> {
    match cursor.peek() {
        Some(value) if is_numbering_spec(value) => cursor.take_next(label),
        Some(_) | None => Ok(default.to_owned()),
    }
}

pub fn take_long_any_or_default(
    cursor: &mut Cursor<'_>,
    default: &str,
    label: &str,
) -> Result<String, Error> {
    match cursor.peek() {
        Some(_) => cursor.take_next(label),
        None => Ok(default.to_owned()),
    }
}

/// Exact match wins; otherwise fall back to unambiguous-prefix inference.
pub fn resolve_long<'a, S: Storage>(
    name: &str,
    flags: &'a [FlagDef<S>],
    exact: bool,
) -> LongMatch<'a, S> {
    if !name.is_empty()
        && let Some(def) = flags.iter().find(|f| {
            f.long.as_ref() == name
                || f.aliases
                    .as_ref()
                    .iter()
                    .any(|alias| alias.as_ref() == name)
        })
    {
        return LongMatch::Flag(def);
    }
    match name {
        "help" => LongMatch::Help,
        "version" => LongMatch::Version,
        _ if exact => LongMatch::Unknown,
        _ => infer_long(name, flags),
    }
}

/// Every long name a flag answers to: its primary long plus any aliases.
pub fn long_names<S: Storage>(flag: &FlagDef<S>) -> impl Iterator<Item = &str> {
    core::iter::once(flag.long.as_ref())
        .chain(flag.aliases.as_ref().iter().map(AsRef::as_ref))
        .chain(
            flag.visible_aliases
                .as_ref()
                .iter()
                .map(AsRef::as_ref)
                // a one-character alias names a short flag, so it is never a long option
                .filter(|alias| alias.chars().count() > 1),
        )
        .filter(|n| !n.is_empty())
}

/// Collect every declared long (plus reserved help/version) prefixed by `name`.
pub fn infer_long<'a, S: Storage>(name: &str, flags: &'a [FlagDef<S>]) -> LongMatch<'a, S> {
    if name.is_empty() {
        return LongMatch::Unknown;
    }
    let mut found = LongMatch::Unknown;
    let mut count = 0_u32;
    for def in flags {
        // A flag matched by several of its own names (long + alias) still counts once.
        if long_names(def).any(|n| n.starts_with(name)) {
            found = LongMatch::Flag(def);
            count = count.saturating_add(1);
        }
    }
    for (reserved, hit) in [("help", LongMatch::Help), ("version", LongMatch::Version)] {
        // A flag that owns this exact long name shadows the reserved option.
        if reserved.starts_with(name) && !flags.iter().any(|f| long_names(f).any(|n| n == reserved))
        {
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
