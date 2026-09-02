//! Pulling a value out of a cluster, an `=`, or the next argument

use crate::def::{Flag, ValueMode};
use crate::error::Error;

use super::Spelling;
use super::config::ScanConfig;
use super::cursor::Cursor;
use super::long::{LongMatch, resolve_long};
use super::numeric::is_numbering_spec;

/// Whether a flag was spelled short, and then whether it stood alone in its cluster
#[derive(Clone, Copy)]
pub enum Form<'a> {
    Short { exact: bool, cluster: &'a str },
    Long,
}

/// The value one valued flag takes, checked against the values it accepts
pub fn take_value<'a>(
    flag: &Flag,
    attached: Option<&'a str>,
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a>,
    spelling: Spelling<'a>,
    form: Form<'a>,
) -> Result<&'a str, Error> {
    let value = pick_value(flag, attached, cursor, config, spelling, form)?;
    if flag.accepted_values().len() > 0 && !flag.accepted_values().any(|accepted| accepted == value)
    {
        let accepted: Vec<&str> = flag.accepted_values().collect();
        return Err(Error::InvalidValue {
            flag: spelling.to_string(),
            value: value.to_owned(),
            reason: format!("expected one of {}", accepted.join(", ")),
        });
    }
    Ok(value)
}

/// The value from what was attached, or from the cursor under the flag's value rule
fn pick_value<'a>(
    flag: &Flag,
    attached: Option<&'a str>,
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a>,
    spelling: Spelling<'a>,
    form: Form<'a>,
) -> Result<&'a str, Error> {
    let ch = flag.id();
    let policy = config.policy;
    if let Some(value) = attached {
        return match form {
            Form::Short { cluster, .. } if policy.requires_separated(ch) => {
                Err(Error::UnknownFlag(Spelling::Arg(cluster).to_string()))
            }
            Form::Short { .. } => Ok(value.strip_prefix('=').unwrap_or(value)),
            Form::Long => Ok(value),
        };
    }
    match form {
        Form::Short { .. } if policy.requires_attached(ch) => {
            return Err(Error::MissingValue(spelling.to_string()));
        }
        Form::Long if policy.requires_equals(ch) => {
            return Err(Error::UnknownFlag(spelling.to_string()));
        }
        _ => {}
    }
    let Some(rule) = policy.value(ch) else {
        return take_plain(flag, cursor, config, spelling);
    };
    let default = rule.default.as_ref();
    match rule.mode {
        ValueMode::AttachedOrDefault => Ok(default),
        ValueMode::NextOrDefault => Ok(cursor
            .take_if(|value| !is_option_boundary(value, config))
            .unwrap_or(default)),
        ValueMode::NumericNextOrDefault => match (form, cursor.peek()) {
            (Form::Long, _) | (_, None) => cursor
                .take()
                .ok_or_else(|| Error::MissingValue(spelling.to_string())),
            (Form::Short { .. }, Some(value)) if is_numbering_spec(value) => {
                cursor.advance();
                Ok(value)
            }
            (Form::Short { .. }, Some(_)) => Ok(default),
        },
        ValueMode::OptionalNumericNextOrDefault => {
            Ok(cursor.take_if(is_numbering_spec).unwrap_or(default))
        }
        ValueMode::AnyNextOrDefault => Ok(cursor.take().unwrap_or(default)),
        ValueMode::ExactShortDefault => match form {
            Form::Short { exact: true, .. } => Ok(default),
            Form::Short { .. } => take_past_boundary(cursor, config, spelling),
            Form::Long => take_plain(flag, cursor, config, spelling),
        },
    }
}

/// The next argument unless it reads as an option, which leaves the value missing
fn take_past_boundary<'a>(
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a>,
    spelling: Spelling<'a>,
) -> Result<&'a str, Error> {
    if cursor
        .peek()
        .is_some_and(|value| is_option_boundary(value, config))
    {
        return Err(Error::MissingValue(spelling.to_string()));
    }
    cursor
        .take()
        .ok_or_else(|| Error::MissingValue(spelling.to_string()))
}

/// The next argument as the value, after refusing a declared option in its place
fn take_plain<'a>(
    flag: &Flag,
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a>,
    spelling: Spelling<'a>,
) -> Result<&'a str, Error> {
    reject_option_value(cursor.peek(), flag, config, spelling)?;
    cursor
        .take()
        .ok_or_else(|| Error::MissingValue(spelling.to_string()))
}

fn reject_option_value(
    value: Option<&str>,
    flag: &Flag,
    config: &ScanConfig<'_>,
    spelling: Spelling<'_>,
) -> Result<(), Error> {
    if flag.allows_hyphen_values() {
        return Ok(());
    }
    let Some(value) = value.filter(|value| value.starts_with('-') && value.len() > 1) else {
        return Ok(());
    };
    if is_declared_option(value, config) {
        Err(Error::MissingValue(spelling.to_string()))
    } else {
        Err(Error::UnknownFlag(value.to_owned()))
    }
}

fn is_declared_option(value: &str, config: &ScanConfig<'_>) -> bool {
    if value == "--" {
        return true;
    }
    if let Some(name) = value.strip_prefix("--") {
        let name = name.split_once('=').map_or(name, |(name, _)| name);
        return !matches!(resolve_long(name, config), LongMatch::Unknown);
    }
    value
        .strip_prefix('-')
        .and_then(|cluster| cluster.chars().next())
        .is_some_and(|ch| config.find(ch).is_some() || matches!(ch, 'h' | 'V'))
}

/// An argument that opens like an option, so a value taken from it would swallow a flag
pub fn is_option_boundary(value: &str, config: &ScanConfig<'_>) -> bool {
    (value.starts_with('-') && value.len() > 1)
        || value
            .strip_prefix('+')
            .and_then(|cluster| cluster.chars().next())
            .is_some_and(|ch| config.find(ch).is_some())
}
