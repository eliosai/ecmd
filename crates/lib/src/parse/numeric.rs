//! Obsolete numeric options such as `-5` and `+3:7`

use crate::error::Error;
use crate::meta::Storage;

use super::config::{ScanConfig, find_flag, parsed_char, record_unimplemented};
use super::cursor::Cursor;
use super::reject::{reject_latest_conflict, reject_repeat};
use super::{Parsed, ScanResult};

pub fn process_first_numeric<S: Storage, P: Storage>(
    arg: &str,
    cursor: &mut Cursor<'_>,
    config: &ScanConfig<'_, S, P>,
    result: &mut ScanResult,
) -> Result<bool, Error> {
    let Some(ch) = config.policy.first_numeric_value else {
        return Ok(false);
    };
    let Some(value) = first_numeric_value(arg) else {
        return Ok(false);
    };
    let Some(def) = find_flag(ch, config.flags) else {
        return Ok(false);
    };
    if first_numeric_is_operand(value) {
        result.operands.push(arg.to_owned());
    } else if !cursor.at_start() {
        let option = value.chars().next().unwrap_or_default();
        return Err(Error::FirstNumericValue {
            option,
            flag: ch,
            value_name: def.value_name.as_ref().to_owned(),
        });
    } else {
        record_unimplemented(def, arg, result);
        reject_repeat(def, result, arg, config.style)?;
        result.flags.push(Parsed::Value(ch, value.to_owned()));
        reject_latest_conflict(result, config, arg)?;
    }
    cursor.advance();
    Ok(true)
}

pub fn first_numeric_value(arg: &str) -> Option<&str> {
    arg.strip_prefix('-')
        .filter(|value| value.as_bytes().first().is_some_and(u8::is_ascii_digit))
}

pub fn first_numeric_is_operand(value: &str) -> bool {
    value.bytes().all(|byte| byte == b'0')
        || (is_unsigned(value) && value.parse::<usize>().is_err())
}

pub fn process_numeric_operand<S: Storage, P: Storage>(
    arg: &str,
    cursor: &mut Cursor<'_>,
    config: &ScanConfig<'_, S, P>,
    result: &mut ScanResult,
) -> Result<bool, Error> {
    let Some((rule, value)) = config.policy.numeric_operands.iter().find_map(|rule| {
        arg.strip_prefix(rule.prefix)
            .filter(|value| is_numeric_range(value))
            .map(|value| (rule, value))
    }) else {
        return Ok(false);
    };
    let Some(def) = find_flag(rule.ch, config.flags) else {
        return Ok(false);
    };
    if result
        .flags
        .iter()
        .any(|parsed| parsed_char(parsed) == rule.ch)
    {
        cursor.advance();
        return Ok(true);
    }
    record_unimplemented(def, arg, result);
    reject_repeat(def, result, arg, config.style)?;
    result.flags.push(Parsed::Value(rule.ch, value.to_owned()));
    reject_latest_conflict(result, config, arg)?;
    cursor.advance();
    Ok(true)
}

pub fn is_numeric_range(value: &str) -> bool {
    let Some((first, last)) = value.split_once(':') else {
        return is_unsigned(value);
    };
    is_unsigned(first) && is_unsigned(last) && !last.contains(':')
}

pub fn is_unsigned(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}
