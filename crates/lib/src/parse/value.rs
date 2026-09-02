//! Pulling a value out of a cluster or the next argument

use crate::error::Error;
use crate::meta::Storage;
use crate::policy::{ValueMode, ValueRule};

use super::FlagDef;
use super::config::{ScanConfig, find_flag};
use super::cursor::Cursor;
use super::long::{LongMatch, resolve_long};

pub fn extract_value<S: Storage, P: Storage>(
    chars: &str,
    byte_pos: usize,
    cursor: &mut Cursor<'_>,
    flag_ch: char,
    def: &FlagDef<S>,
    config: &ScanConfig<'_, S, P>,
) -> Result<String, Error> {
    let after = byte_pos.saturating_add(1);
    let remainder = chars.get(after..).unwrap_or_default();
    if !remainder.is_empty() {
        if config.policy.requires_separated(flag_ch) {
            return Err(Error::UnknownFlag(format!("-{chars}")));
        }
        cursor.advance();
        // clap parity: a single leading `=` is the attached-value separator (`-c=5` → `5`).
        return Ok(remainder.strip_prefix('=').unwrap_or(remainder).to_owned());
    }
    let label = format!("-{flag_ch}");
    if config.policy.requires_attached(flag_ch) {
        return Err(Error::MissingValue(label));
    }
    if let Some(rule) = config.policy.value(flag_ch) {
        return apply_short_value_rule(rule, cursor, config.flags, &label, chars.len() == 1);
    }
    reject_option_value(
        cursor.following(),
        def,
        config.flags,
        config.policy.exact_long,
        &label,
    )?;
    cursor.next_value(flag_ch)
}

pub fn apply_short_value_rule<S: Storage, P: Storage>(
    rule: &ValueRule<P>,
    cursor: &mut Cursor<'_>,
    flags: &[FlagDef<S>],
    label: &str,
    exact: bool,
) -> Result<String, Error> {
    match rule.mode {
        ValueMode::AttachedOrDefault => Ok(advance_with_default(cursor, rule.default.as_ref())),
        ValueMode::NextOrDefault => Ok(take_next_or_default(cursor, flags, rule.default.as_ref())),
        ValueMode::NumericNextOrDefault => {
            take_numeric_next_or_default(cursor, rule.default.as_ref(), label)
        }
        ValueMode::OptionalNumericNextOrDefault => Ok(take_optional_numeric_next_or_default(
            cursor,
            rule.default.as_ref(),
        )),
        ValueMode::ExactShortDefault if exact => {
            Ok(advance_with_default(cursor, rule.default.as_ref()))
        }
        ValueMode::ExactShortDefault => take_non_option_next(cursor, flags, label, rule.ch),
        ValueMode::AnyNextOrDefault => Ok(take_any_next_or_default(cursor, rule.default.as_ref())),
    }
}

pub fn take_non_option_next<S: Storage>(
    cursor: &mut Cursor<'_>,
    flags: &[FlagDef<S>],
    label: &str,
    ch: char,
) -> Result<String, Error> {
    if cursor
        .following()
        .is_some_and(|value| is_option_boundary(value, flags))
    {
        return Err(Error::MissingValue(label.to_owned()));
    }
    cursor.next_value(ch)
}

pub fn take_any_next_or_default(cursor: &mut Cursor<'_>, default: &str) -> String {
    match cursor.following() {
        Some(value) => {
            cursor.advance();
            cursor.advance();
            value.to_owned()
        }
        None => advance_with_default(cursor, default),
    }
}

pub fn advance_with_default(cursor: &mut Cursor<'_>, default: &str) -> String {
    cursor.advance();
    default.to_owned()
}

pub fn take_next_or_default<S: Storage>(
    cursor: &mut Cursor<'_>,
    flags: &[FlagDef<S>],
    default: &str,
) -> String {
    if let Some(value) = cursor
        .following()
        .filter(|value| !is_option_boundary(value, flags))
    {
        cursor.advance();
        cursor.advance();
        return value.to_owned();
    }
    advance_with_default(cursor, default)
}

pub fn take_numeric_next_or_default(
    cursor: &mut Cursor<'_>,
    default: &str,
    label: &str,
) -> Result<String, Error> {
    match cursor.following() {
        Some(value) if is_numbering_spec(value) => {
            cursor.advance();
            cursor.take_next(label)
        }
        Some(_) => Ok(advance_with_default(cursor, default)),
        None => Err(Error::MissingValue(label.to_owned())),
    }
}

pub fn take_optional_numeric_next_or_default(cursor: &mut Cursor<'_>, default: &str) -> String {
    match cursor.following() {
        Some(value) if is_numbering_spec(value) => {
            let value = value.to_owned();
            cursor.advance();
            cursor.advance();
            value
        }
        Some(_) | None => advance_with_default(cursor, default),
    }
}

pub fn reject_option_value<S: Storage>(
    value: Option<&str>,
    def: &FlagDef<S>,
    flags: &[FlagDef<S>],
    exact_long: bool,
    label: &str,
) -> Result<(), Error> {
    if def.allow_hyphen_values {
        return Ok(());
    }
    let Some(value) = value.filter(|value| value.starts_with('-') && value.len() > 1) else {
        return Ok(());
    };
    if is_declared_option(value, flags, exact_long) {
        Err(Error::MissingValue(label.to_owned()))
    } else {
        Err(Error::UnknownFlag(value.to_owned()))
    }
}

pub fn is_declared_option<S: Storage>(value: &str, flags: &[FlagDef<S>], exact_long: bool) -> bool {
    if value == "--" {
        return true;
    }
    if let Some(name) = value.strip_prefix("--") {
        let name = name.split_once('=').map_or(name, |(name, _)| name);
        return !matches!(resolve_long(name, flags, exact_long), LongMatch::Unknown);
    }
    value
        .strip_prefix('-')
        .and_then(|cluster| cluster.chars().next())
        .is_some_and(|ch| find_flag(ch, flags).is_some() || matches!(ch, 'h' | 'V'))
}

pub fn is_option_boundary<S: Storage>(value: &str, flags: &[FlagDef<S>]) -> bool {
    (value.starts_with('-') && value.len() > 1)
        || value
            .strip_prefix('+')
            .and_then(|cluster| cluster.chars().next())
            .is_some_and(|ch| find_flag(ch, flags).is_some())
}

pub fn is_numbering_spec(value: &str) -> bool {
    if value.starts_with('-') && value.len() > 1 {
        return true;
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii() && chars.all(|ch| ch.is_ascii_digit())
}
