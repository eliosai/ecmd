//! Sorting one argument into a flag cluster or an operand

use crate::polarity::Polarity;

use super::FlagKind;

pub const fn is_polar(kind: &FlagKind) -> bool {
    matches!(kind, FlagKind::Polar | FlagKind::PolarValue)
}

pub enum ArgClass<'a> {
    Flags(Polarity, &'a str),
    Operand,
}

pub fn classify(arg: &str, has_polarity: bool) -> ArgClass<'_> {
    let Some((&first, rest)) = arg.as_bytes().split_first() else {
        return ArgClass::Operand;
    };
    if rest.is_empty() {
        return ArgClass::Operand;
    }
    let chars = arg.get(1..).unwrap_or_default();
    match first {
        b'-' => ArgClass::Flags(Polarity::On, chars),
        b'+' if has_polarity => ArgClass::Flags(Polarity::Off, chars),
        _ => ArgClass::Operand,
    }
}
