//! Derive macro for ecmd `Command` trait.
//!
//! Generates `parse()` and metadata from struct definitions.
//! Field types drive parsing behavior automatically.

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod attrs;
mod classify;
mod codegen;

/// Derive the `Command` trait for a struct.
///
/// # Attributes
///
/// Struct-level: `#[command(name = "cmd", style = "posix")]`
/// Field-level: `#[flag(short = 'v')]`
///
/// Fields without `#[flag]` are positionals (by order).
/// Fields of type `Operands` consume all remaining args.
#[proc_macro_derive(Command, attributes(command, flag))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    codegen::expand(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
