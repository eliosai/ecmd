#![doc = include_str!("../README.md")]

#[cfg(feature = "rkyv")]
mod archive;
mod def;
mod error;
mod help;
mod operands;
mod parse;
mod polarity;
mod style;

#[doc(hidden)]
pub mod __private;

pub use def::{Def, DefBuilder, Flag, FlagKind, Positional};
pub use error::Error;
pub use operands::Operands;
pub use parse::{Parsed, Scan, Spelling};
pub use polarity::{PolarVal, Polarity};
pub use style::{HelpStyle, Style};

#[cfg(feature = "derive")]
pub use ecmd_derive::Command;

/// A command whose definition and parser `#[derive(Command)]` generates
pub trait Command: Sized {
    /// The static definition of this command
    fn def() -> &'static Def;

    /// Parse one argument slice into the command
    fn parse(args: &[&str]) -> Result<Self, Error>;

    /// Parse the process arguments that follow the program name
    fn parse_env() -> Result<Self, Error> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        Self::parse(&refs)
    }
}
