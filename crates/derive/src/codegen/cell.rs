//! Statics built on first borrow, so a definition carries no load-time relocation

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// A once-built static of `ty` named `name`, and the expression that borrows it
pub fn cell(name: &Ident, ty: &TokenStream, value: &TokenStream) -> (TokenStream, TokenStream) {
    let item = quote! {
        static #name: ::std::sync::OnceLock<#ty> = ::std::sync::OnceLock::new();
    };
    let borrow = quote! {
        #name.get_or_init({
            fn init() -> #ty {
                #value
            }
            init as fn() -> #ty
        })
    };
    (item, borrow)
}

/// A once-built heap slice of `element` holding `values`, or a bare empty slice when there are none
pub fn array_cell(
    name: &Ident,
    element: &TokenStream,
    values: &[TokenStream],
) -> (TokenStream, TokenStream) {
    if values.is_empty() {
        return (TokenStream::new(), quote! { &[] });
    }
    let (item, borrow) = cell(
        name,
        &quote! { ::std::boxed::Box<[#element]> },
        &quote! { ::std::vec![#(#values),*].into_boxed_slice() },
    );
    (item, quote! { &**#borrow })
}

/// A once-built array of borrowed text lines, or a bare empty slice when there are none
pub fn texts_cell(name: &Ident, lines: &[String]) -> (TokenStream, TokenStream) {
    let values: Vec<_> = lines
        .iter()
        .map(|line| quote! { ::std::borrow::Cow::Borrowed(#line) })
        .collect();
    array_cell(name, &quote! { ::std::borrow::Cow<'static, str> }, &values)
}
