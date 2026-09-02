//! Short flag clusters such as `-abc` and `-ofile`

use crate::def::FlagKind;
use crate::error::Error;
use crate::polarity::Polarity;

use super::config::ScanConfig;
use super::cursor::Cursor;
use super::state::State;
use super::value::{Form, take_value};
use super::{Parsed, Spelling};

/// One cluster the cursor has moved past, given as the whole argument and its characters
pub fn process_cluster<'a>(
    arg: &'a str,
    chars: &'a str,
    polarity: Polarity,
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a>,
    state: &mut State<'a>,
) -> Result<(), Error> {
    if process_prefixed_value(arg, chars, polarity, config, state)? {
        return Ok(());
    }
    if config.lenient && has_unknown_flag(chars, config) {
        state.push_operand(arg);
        return Ok(());
    }
    parse_known_cluster(arg, chars, polarity, cursor, config, state)
}

/// A `-5x` form, where the digits before a prefixed-value flag are its value
fn process_prefixed_value<'a>(
    arg: &'a str,
    chars: &'a str,
    polarity: Polarity,
    config: &ScanConfig<'a>,
    state: &mut State<'a>,
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
    let Some((position, flag)) = config.find(ch) else {
        return Ok(false);
    };
    let value = chars.get(..prefix.len()).unwrap_or_default();
    state.record(
        config,
        position,
        flag,
        Parsed::Value(ch, value),
        Spelling::Arg(arg),
    )?;
    Ok(true)
}

fn has_unknown_flag(chars: &str, config: &ScanConfig<'_>) -> bool {
    for (index, &byte) in chars.as_bytes().iter().enumerate() {
        let ch = char::from(byte);
        match config.find(ch) {
            None if implicit_short_action(ch, config).is_some() => {}
            None => return true,
            Some((_, flag)) if flag.takes_value() => {
                let attached = index.saturating_add(1) < chars.len();
                return attached && config.policy.requires_separated(flag.id());
            }
            Some(_) => {}
        }
    }
    false
}

fn parse_known_cluster<'a>(
    arg: &'a str,
    chars: &'a str,
    polarity: Polarity,
    cursor: &mut Cursor<'a>,
    config: &ScanConfig<'a>,
    state: &mut State<'a>,
) -> Result<(), Error> {
    for (index, &byte) in chars.as_bytes().iter().enumerate() {
        let ch = char::from(byte);
        let spelling = Spelling::Short(ch);
        let Some((position, flag)) = config.find(ch) else {
            return Err(implicit_short_action(ch, config)
                .unwrap_or_else(|| Error::UnknownFlag(spelling.to_string())));
        };
        let parsed = match flag.flag_kind() {
            FlagKind::Bool | FlagKind::Noop => Parsed::Bool(ch),
            FlagKind::Polar => Parsed::Polar(ch, polarity),
            FlagKind::Value | FlagKind::PolarValue => {
                let attached = chars
                    .get(index.saturating_add(1)..)
                    .filter(|rest| !rest.is_empty());
                let form = Form::Short {
                    exact: chars.len() == 1,
                    cluster: arg,
                };
                let value = take_value(flag, attached, cursor, config, spelling, form)?;
                let parsed = if flag.flag_kind() == FlagKind::Value {
                    Parsed::Value(ch, value)
                } else {
                    Parsed::PolarValue(ch, polarity, value)
                };
                return state.record(config, position, flag, parsed, spelling);
            }
        };
        state.record(config, position, flag, parsed, spelling)?;
    }
    Ok(())
}

/// The action a GNU command reserves for an undeclared `-h` or `-V`
pub fn implicit_short_action(ch: char, config: &ScanConfig<'_>) -> Option<Error> {
    if !config.gnu() {
        return None;
    }
    match ch {
        'h' => Some(Error::HelpRequested),
        'V' if !config.policy.no_implicit_version => Some(Error::VersionRequested),
        _ => None,
    }
}
