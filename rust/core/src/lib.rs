//! Type-driven command-line argument parsing.
//!
//! ecmd generates parsers from struct definitions. Field types drive behavior:
//! `bool` becomes a flag, `Option<T>` an optional value, `Operands` the rest.
//!
//! ```rust
//! use ecmd::Command;
//! use ecmd::meta::Command as _;
//! use ecmd::operands::Operands;
//!
//! #[derive(Command)]
//! #[command(name = "grep", style = "posix")]
//! struct Grep {
//!     #[flag(short = 'i')]
//!     ignore_case: bool,
//!     #[flag(short = 'n')]
//!     line_numbers: bool,
//!     pattern: String,
//!     files: Operands,
//! }
//!
//! let cmd = Grep::parse(&["-i", "hello", "src/main.rs"]).unwrap();
//! assert!(cmd.ignore_case);
//! assert_eq!(cmd.pattern, "hello");
//! assert_eq!(cmd.files.first(), Some("src/main.rs"));
//! ```

pub mod error;
pub mod meta;
pub mod operands;
pub mod parse;
pub mod polarity;
pub mod prelude;
pub mod style;

/// Derive macro for the `Command` trait.
#[cfg(feature = "derive")]
pub use ecmd_derive::Command;
