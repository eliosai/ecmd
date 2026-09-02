//! What every scanning step shares

use crate::meta::Storage;
use crate::style::Style;

use super::{FlagDef, OnUnknown, Parsed, Policy, ScanResult};

pub struct ScanConfig<'a, S: Storage, P: Storage> {
    pub flags: &'a [FlagDef<S>],
    pub policy: Policy<'a, P>,
    pub on_unknown: OnUnknown,
    pub style: Style,
}

pub fn record_unimplemented<S: Storage>(def: &FlagDef<S>, label: &str, result: &mut ScanResult) {
    if !def.implemented {
        result.unimplemented.push(label.to_owned());
    }
}

/// A `--name` or `--name=value` token (but not the bare `--` terminator).
pub fn is_long(arg: &str) -> bool {
    arg.len() > 2 && arg.starts_with("--")
}

pub fn find_flag<S: Storage>(ch: char, flags: &[FlagDef<S>]) -> Option<&FlagDef<S>> {
    flags.iter().find(|f| f.ch == ch)
}

pub const fn parsed_char(parsed: &Parsed) -> char {
    match parsed {
        Parsed::Bool(ch)
        | Parsed::Value(ch, _)
        | Parsed::Polar(ch, _)
        | Parsed::PolarValue(ch, _, _) => *ch,
    }
}
