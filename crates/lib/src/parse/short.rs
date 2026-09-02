//! Short flag clusters such as `-abc` and `-ofile`

use crate::error::Error;
use crate::meta::Storage;
use crate::polarity::Polarity;
use crate::style::Style;

use super::config::{ScanConfig, find_flag, record_unimplemented};
use super::cursor::Cursor;
use super::reject::{reject_latest_conflict, reject_repeat};
use super::value::extract_value;
use super::{FlagDef, FlagKind, OnUnknown, Parsed, ScanResult};

pub fn process_cluster<S: Storage, P: Storage>(
    chars: &str,
    polarity: Polarity,
    cursor: &mut Cursor<'_>,
    config: &ScanConfig<'_, S, P>,
    result: &mut ScanResult,
) -> Result<(), Error> {
    if process_prefixed_value(chars, polarity, cursor, config, result)? {
        return Ok(());
    }
    if try_passthrough(chars, cursor, config, &mut result.operands) {
        return Ok(());
    }
    parse_known_cluster(chars, polarity, cursor, config, result)
}

pub fn process_prefixed_value<S: Storage, P: Storage>(
    chars: &str,
    polarity: Polarity,
    cursor: &mut Cursor<'_>,
    config: &ScanConfig<'_, S, P>,
    result: &mut ScanResult,
) -> Result<bool, Error> {
    let Some((last, prefix)) = chars.as_bytes().split_last() else {
        return Ok(false);
    };
    let ch = char::from(*last);
    if polarity != Polarity::On
        || prefix.is_empty()
        || !prefix.iter().all(u8::is_ascii_digit)
        || !config.policy.accepts_prefix(ch)
    {
        return Ok(false);
    }
    let Some(def) = find_flag(ch, config.flags) else {
        return Ok(false);
    };
    let value = prefix.iter().copied().map(char::from).collect();
    let label = format!("-{chars}");
    record_unimplemented(def, &label, result);
    reject_repeat(def, result, &label, config.style)?;
    result.flags.push(Parsed::Value(ch, value));
    reject_latest_conflict(result, config, &label)?;
    cursor.advance();
    Ok(true)
}

pub fn try_passthrough<S: Storage, P: Storage>(
    chars: &str,
    cursor: &mut Cursor<'_>,
    config: &ScanConfig<'_, S, P>,
    operands: &mut Vec<String>,
) -> bool {
    if config.on_unknown != OnUnknown::PassThrough {
        return false;
    }
    if !has_unknown_flag(chars, config) {
        return false;
    }
    if let Some(full_arg) = cursor.peek() {
        operands.push(full_arg.to_owned());
    }
    cursor.advance();
    true
}

pub fn has_unknown_flag<S: Storage, P: Storage>(
    chars: &str,
    config: &ScanConfig<'_, S, P>,
) -> bool {
    for (index, &b) in chars.as_bytes().iter().enumerate() {
        match find_flag(char::from(b), config.flags) {
            None if implicit_short_action(char::from(b), config.style, config.flags).is_some() => {}
            None => return true,
            Some(def) if matches!(def.kind, FlagKind::Value | FlagKind::PolarValue) => {
                if index.saturating_add(1) < chars.len() && config.policy.requires_separated(def.ch)
                {
                    return true;
                }
                return false;
            }
            _ => {}
        }
    }
    false
}

pub fn parse_known_cluster<S: Storage, P: Storage>(
    chars: &str,
    polarity: Polarity,
    cursor: &mut Cursor<'_>,
    config: &ScanConfig<'_, S, P>,
    result: &mut ScanResult,
) -> Result<(), Error> {
    for (bi, &byte) in chars.as_bytes().iter().enumerate() {
        let ch = char::from(byte);
        let Some(def) = find_flag(ch, config.flags) else {
            return Err(implicit_short_action(ch, config.style, config.flags)
                .unwrap_or_else(|| Error::UnknownFlag(format!("-{ch}"))));
        };
        if !def.implemented {
            result.unimplemented.push(format!("-{ch}"));
        }
        match def.kind {
            FlagKind::Bool | FlagKind::Noop => {
                reject_repeat(def, result, &format!("-{ch}"), config.style)?;
                result.flags.push(Parsed::Bool(ch));
                reject_latest_conflict(result, config, &format!("-{ch}"))?;
            }
            FlagKind::Polar => {
                reject_repeat(def, result, &format!("-{ch}"), config.style)?;
                result.flags.push(Parsed::Polar(ch, polarity));
                reject_latest_conflict(result, config, &format!("-{ch}"))?;
            }
            FlagKind::Value | FlagKind::PolarValue => {
                let value = extract_value(chars, bi, cursor, ch, def, config)?;
                reject_repeat(def, result, &format!("-{ch}"), config.style)?;
                let parsed = match def.kind {
                    FlagKind::Value => Parsed::Value(ch, value),
                    _ => Parsed::PolarValue(ch, polarity, value),
                };
                result.flags.push(parsed);
                reject_latest_conflict(result, config, &format!("-{ch}"))?;
                return Ok(());
            }
        }
    }
    cursor.advance();
    Ok(())
}

pub fn implicit_short_action<S: Storage>(
    ch: char,
    style: Style,
    flags: &[FlagDef<S>],
) -> Option<Error> {
    if style != Style::Gnu {
        return None;
    }
    match ch {
        'h' => Some(Error::HelpRequested),
        'V' if !no_implicit_version(flags) => Some(Error::VersionRequested),
        _ => None,
    }
}

pub fn no_implicit_version<S: Storage>(flags: &[FlagDef<S>]) -> bool {
    flags
        .iter()
        .any(|flag| flag.hidden && flag.long.as_ref() == "\0no-implicit-version")
}
