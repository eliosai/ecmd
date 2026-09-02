//! Derive macro for the ecmd `Command` trait

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod attrs;
mod classify;
mod codegen;

/// Derive the `Command` trait from a struct's field types and its `command`, `flag` and `operand` attributes
#[proc_macro_derive(Command, attributes(command, flag, operand))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    codegen::expand(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
