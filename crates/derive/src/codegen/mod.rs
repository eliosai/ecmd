//! Code generation for the `Command` derive.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, Ident};

mod meta;
mod parse;
mod policy;
mod validate;

use meta::gen_meta;
use parse::gen_parse;
use policy::gen_policy;
use validate::validate;

use crate::attrs::{
    CommandAttrs, FlagAttrs, OperandAttrs, extract_doc_comment, extract_doc_sections,
};
use crate::classify::{FieldRole, classify_field, field_ident};

/// Classified field with its role pre-computed.
pub struct ClassifiedField<'a> {
    pub field: &'a Field,
    pub ident: &'a Ident,
    pub role: FieldRole,
    pub desc: String,
    /// Identity char: the short flag, or a private-use codepoint for long-only.
    pub id: char,
    /// Help presentation for a positional or rest field.
    pub operand: OperandAttrs,
}

/// Main expansion entry point.
pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let cmd = CommandAttrs::from_ast(&input.attrs)?;
    let sections = extract_doc_sections(&input.attrs);
    let raw_fields = extract_named_fields(input)?;
    let fields = classify_all(raw_fields)?;
    validate(&fields)?;
    let name = &input.ident;

    let policy = gen_policy(&cmd, &fields)?;
    let meta_body = gen_meta(&cmd, &sections, &fields, &policy);
    let parse_body = gen_parse(&fields);

    Ok(quote! {
        impl ::ecmd::Command for #name {
            fn def() -> &'static ::ecmd::Def {
                #meta_body
            }

            fn parse(args: &[&str]) -> ::core::result::Result<Self, ::ecmd::Error> {
                #parse_body
            }
        }
    })
}

fn extract_named_fields(
    input: &DeriveInput,
) -> syn::Result<&syn::punctuated::Punctuated<Field, syn::token::Comma>> {
    match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => Ok(&n.named),
            _ => Err(syn::Error::new_spanned(input, "only named structs")),
        },
        _ => Err(syn::Error::new_spanned(input, "only structs")),
    }
}

fn classify_all(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> syn::Result<Vec<ClassifiedField<'_>>> {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let role = classify_field(f)?;
            let desc = extract_doc_comment(&f.attrs);
            let id = field_id(&role, i);
            Ok(ClassifiedField {
                field: f,
                ident: field_ident(f),
                role,
                desc,
                id,
                operand: OperandAttrs::from_field(f)?,
            })
        })
        .collect()
}

/// A flag's identity char: its short, or a private-use codepoint for long-only.
fn field_id(role: &FieldRole, index: usize) -> char {
    match flag_attrs(role) {
        Some(a) if a.short != '\0' => a.short,
        Some(_) => char::from_u32(0xE000_u32.saturating_add(u32::try_from(index).unwrap_or(0)))
            .unwrap_or('\u{E000}'),
        None => '\0',
    }
}

pub const fn flag_attrs(role: &FieldRole) -> Option<&FlagAttrs> {
    match role {
        FieldRole::BoolFlag(a)
        | FieldRole::PolarityFlag(a)
        | FieldRole::ValuedFlag(a)
        | FieldRole::RepeatableValueFlag(a)
        | FieldRole::PolarValueFlag(a) => Some(a),
        _ => None,
    }
}

pub fn clears_targets(role: &FieldRole) -> &[Ident] {
    match role {
        FieldRole::BoolFlag(a)
        | FieldRole::PolarityFlag(a)
        | FieldRole::ValuedFlag(a)
        | FieldRole::RepeatableValueFlag(a)
        | FieldRole::PolarValueFlag(a) => &a.clears,
        _ => &[],
    }
}

pub fn flag_def_tokens(role: &FieldRole) -> Option<(char, TokenStream)> {
    match role {
        FieldRole::BoolFlag(a) => Some((a.short, quote! { ::ecmd::FlagKind::Bool })),
        FieldRole::PolarityFlag(a) => Some((a.short, quote! { ::ecmd::FlagKind::Polar })),
        FieldRole::ValuedFlag(a) | FieldRole::RepeatableValueFlag(a) => {
            Some((a.short, quote! { ::ecmd::FlagKind::Value }))
        }
        FieldRole::PolarValueFlag(a) => Some((a.short, quote! { ::ecmd::FlagKind::PolarValue })),
        _ => None,
    }
}

pub fn flag_value_name(role: &FieldRole) -> &str {
    match role {
        FieldRole::ValuedFlag(a)
        | FieldRole::RepeatableValueFlag(a)
        | FieldRole::PolarValueFlag(a) => {
            if a.value_name.is_empty() {
                "ARG"
            } else {
                &a.value_name
            }
        }
        _ => "",
    }
}
