//! Repeats and exclusive groups a style refuses

use crate::error::Error;
use crate::meta::Storage;
use crate::style::Style;

use super::config::{ScanConfig, find_flag, parsed_char};
use super::{FlagDef, ScanResult};

pub fn reject_repeat<S: Storage>(
    def: &FlagDef<S>,
    result: &ScanResult,
    label: &str,
    style: Style,
) -> Result<(), Error> {
    if style != Style::Gnu
        || def.repeatable
        || !result.flags.iter().any(|flag| parsed_char(flag) == def.ch)
    {
        return Ok(());
    }
    Err(Error::RepeatedFlag(label.to_owned()))
}

pub fn reject_latest_conflict<S: Storage, P: Storage>(
    result: &ScanResult,
    config: &ScanConfig<'_, S, P>,
    current: &str,
) -> Result<(), Error> {
    let Some(current_ch) = result.flags.last().map(parsed_char) else {
        return Ok(());
    };
    let Some(current_group) = config.policy.exclusive_group(current_ch) else {
        return Ok(());
    };
    let previous = result.flags.iter().rev().skip(1).find_map(|parsed| {
        let ch = parsed_char(parsed);
        config
            .policy
            .exclusive_group(ch)
            .filter(|group| *group != current_group)
            .and_then(|_| find_flag(ch, config.flags))
    });
    previous.map_or(Ok(()), |def| {
        Err(Error::ConflictingFlags {
            current: current.to_owned(),
            previous: preferred_label(def),
        })
    })
}

pub fn preferred_label<S: Storage>(def: &FlagDef<S>) -> String {
    if def.ch < '\u{E000}' {
        format!("-{}", def.ch)
    } else {
        format!("--{}", def.long.as_ref())
    }
}
