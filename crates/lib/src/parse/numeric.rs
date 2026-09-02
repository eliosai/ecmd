//! Obsolete numeric options such as `-5` and `+3:7`

use crate::error::Error;
use crate::meta::Storage;

use super::config::ScanConfig;
use super::cursor::Cursor;
use super::state::State;
use super::{Parsed, Spelling};

/// A leading `-N` routed to the first-numeric flag, an operand when it is no count
pub fn process_first_numeric<'a, S: Storage, P: Storage>(
    arg: &'a str,
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a, S, P>,
    state: &mut State<'a>,
) -> Result<bool, Error> {
    let Some(ch) = config.policy.first_numeric_value else {
        return Ok(false);
    };
    let Some(value) = first_numeric_value(arg) else {
        return Ok(false);
    };
    let Some((position, def)) = config.find(ch) else {
        return Ok(false);
    };
    if first_numeric_is_operand(value) {
        state.push_operand(arg);
    } else if !cursor.at_start() {
        return Err(Error::FirstNumericValue {
            option: value.chars().next().unwrap_or_default(),
            flag: ch,
            value_name: def.value_name.as_ref().to_owned(),
        });
    } else {
        state.record(
            config,
            position,
            def,
            Parsed::Value(ch, value),
            Spelling::Arg(arg),
        )?;
    }
    cursor.advance();
    Ok(true)
}

fn first_numeric_value(arg: &str) -> Option<&str> {
    arg.strip_prefix('-')
        .filter(|value| value.as_bytes().first().is_some_and(u8::is_ascii_digit))
}

fn first_numeric_is_operand(value: &str) -> bool {
    value.bytes().all(|byte| byte == b'0')
        || (is_unsigned(value) && value.parse::<usize>().is_err())
}

/// A `+N` or `-N:M` operand routed to its valued flag, the first occurrence winning
pub fn process_numeric_operand<'a, S: Storage, P: Storage>(
    arg: &'a str,
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a, S, P>,
    state: &mut State<'a>,
) -> Result<bool, Error> {
    let Some((rule, value)) = config.policy.numeric_operands.iter().find_map(|rule| {
        arg.strip_prefix(rule.prefix)
            .filter(|value| is_numeric_range(value))
            .map(|value| (rule, value))
    }) else {
        return Ok(false);
    };
    let Some((position, def)) = config.find(rule.ch) else {
        return Ok(false);
    };
    cursor.advance();
    if state.seen(position) {
        return Ok(true);
    }
    state.record(
        config,
        position,
        def,
        Parsed::Value(rule.ch, value),
        Spelling::Arg(arg),
    )?;
    Ok(true)
}

fn is_numeric_range(value: &str) -> bool {
    let Some((first, last)) = value.split_once(':') else {
        return is_unsigned(value);
    };
    is_unsigned(first) && is_unsigned(last) && !last.contains(':')
}

fn is_unsigned(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

/// A numbering spec such as `5`, `-3` or `x7` that a numeric value rule accepts
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
