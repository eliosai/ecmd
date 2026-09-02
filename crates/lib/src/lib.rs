//! Type-driven POSIX and GNU argument parsing

#[cfg(feature = "rkyv")]
pub mod archive;
pub mod error;
mod help;
pub mod meta;
pub mod operands;
pub mod parse;
pub mod polarity;
pub mod policy;
pub mod prelude;
pub mod style;

/// Derive macro for the `Command` trait.
#[cfg(feature = "derive")]
pub use ecmd_derive::Command;
