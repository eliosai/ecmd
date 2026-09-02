//! POSIX and GNU argument scanning

use core::fmt;

use crate::def::Def;
use crate::error::Error;
use crate::polarity::Polarity;

mod classify;
mod config;
mod cursor;
mod long;
mod numeric;
mod short;
mod state;
mod value;

use classify::{ArgClass, classify};
use config::{ScanConfig, is_long};
use cursor::Cursor;
use long::process_long;
use numeric::{process_first_numeric, process_numeric_operand};
use short::process_cluster;
use state::State;

/// One flag occurrence the scanner recognized
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Parsed<'a> {
    /// A boolean flag
    Bool(char),
    /// A valued flag with its value
    Value(char, &'a str),
    /// A polarity flag with the sign it carried
    Polar(char, Polarity),
    /// A polarity flag with its sign and value
    PolarValue(char, Polarity, &'a str),
}

impl Parsed<'_> {
    /// The identity of the flag this occurrence names
    #[must_use]
    pub const fn flag(&self) -> char {
        match self {
            Self::Bool(ch)
            | Self::Value(ch, _)
            | Self::Polar(ch, _)
            | Self::PolarValue(ch, _, _) => *ch,
        }
    }
}

/// How the caller spelled one flag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Spelling<'a> {
    /// A short flag such as `-x`
    Short(char),
    /// A long option such as `--name`
    Long(&'a str),
    /// One whole argument such as `-5` or `+3:7`
    Arg(&'a str),
}

impl fmt::Display for Spelling<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Short(ch) => write!(f, "-{ch}"),
            Self::Long(name) => write!(f, "--{name}"),
            Self::Arg(arg) => f.write_str(arg),
        }
    }
}

/// What one scan found: flags in order, operands, and the flags only an external command implements
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Scan<'a> {
    flags: Vec<Parsed<'a>>,
    operands: Vec<&'a str>,
    unimplemented: Vec<Spelling<'a>>,
}

impl<'a> Scan<'a> {
    /// Every flag occurrence in argument order
    #[must_use]
    pub fn flags(&self) -> &[Parsed<'a>] {
        &self.flags
    }

    /// Every operand in argument order
    #[must_use]
    pub fn operands(&self) -> &[&'a str] {
        &self.operands
    }

    /// Every flag that appeared but is declared as implemented elsewhere
    #[must_use]
    pub fn unimplemented(&self) -> &[Spelling<'a>] {
        &self.unimplemented
    }
}

/// Scan arguments against one definition
pub fn scan<'a>(def: &'a Def, args: &'a [&'a str]) -> Result<Scan<'a>, Error> {
    let config = ScanConfig::new(def);
    let mut state = State::new(def.flags().len());
    let mut cursor = Cursor::new(args);
    while let Some(arg) = cursor.peek() {
        if process_first_numeric(arg, &mut cursor, &config, &mut state)?
            || process_numeric_operand(arg, &mut cursor, &config, &mut state)?
        {
            continue;
        }
        if arg == "--" {
            cursor.advance();
            break;
        }
        let operands = state.operand_count();
        if config.gnu() && is_long(arg) {
            cursor.advance();
            let spec = arg.strip_prefix("--").unwrap_or_default();
            if let Err(error) = process_long(spec, &mut cursor, &config, &mut state) {
                state.pass_unknown(error, arg, config.lenient)?;
            }
        } else {
            match classify(arg, config.has_polarity) {
                ArgClass::Flags(polarity, chars) => {
                    cursor.advance();
                    process_cluster(arg, chars, polarity, &mut cursor, &config, &mut state)?;
                }
                ArgClass::Operand if config.gnu() && config.permute => {
                    cursor.advance();
                    state.push_operand(arg);
                }
                ArgClass::Operand => break,
            }
        }
        if !config.permute && state.operand_count() > operands {
            break;
        }
    }
    state.extend_operands(cursor.rest());
    Ok(state.finish())
}
