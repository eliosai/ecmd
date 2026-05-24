//! Type-driven command-line argument parsing.
//!
//! ecmd generates parsers from struct definitions. Field types drive behavior:
//! `bool` becomes a flag, `Option<T>` an optional value, `Operands` the rest.

pub mod error;
pub mod meta;
pub mod operands;
pub mod parse;
pub mod polarity;
pub mod style;
