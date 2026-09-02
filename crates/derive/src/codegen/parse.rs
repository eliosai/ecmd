//! The generated `parse` body

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident};

use super::ClassifiedField;
use crate::classify::FieldRole;

pub fn gen_parse(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let inits = gen_inits(fields);
    let dispatch = gen_dispatch(fields);
    let positionals = gen_positionals(fields);
    let names: Vec<_> = fields.iter().map(|cf| cf.ident).collect();

    quote! {
        let scan = Self::def().scan(args)?;
        if let Some(name) = scan.unimplemented().first() {
            return Err(::ecmd::Error::UnimplementedFlag(name.to_string()));
        }

        #inits

        for flag in scan.flags() {
            match flag {
                #dispatch
                _ => {}
            }
        }

        #positionals

        Ok(Self { #(#names),* })
    }
}

pub fn gen_inits(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let stmts: Vec<_> = fields
        .iter()
        .map(|cf| {
            let id = cf.ident;
            match &cf.role {
                FieldRole::BoolFlag(_) => quote! { let mut #id = false; },
                FieldRole::PolarityFlag(_) => {
                    quote! { let mut #id = ::ecmd::Polarity::Unset; }
                }
                FieldRole::ValuedFlag(_) | FieldRole::OptionalPositional => {
                    quote! { let mut #id = None; }
                }
                FieldRole::PolarValueFlag(_) | FieldRole::RepeatableValueFlag(_) => {
                    quote! { let mut #id = Vec::new(); }
                }
                FieldRole::RequiredPositional | FieldRole::Rest => quote! { let #id; },
            }
        })
        .collect();
    quote! { #(#stmts)* }
}

pub fn gen_dispatch(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let arms: Vec<_> = fields
        .iter()
        .filter_map(|cf| gen_single_dispatch(cf, fields))
        .collect();
    quote! { #(#arms)* }
}

pub fn gen_single_dispatch(
    cf: &ClassifiedField<'_>,
    all: &[ClassifiedField<'_>],
) -> Option<TokenStream> {
    let id = cf.ident;
    match &cf.role {
        FieldRole::BoolFlag(attrs) => {
            let ch = cf.id;
            let resets = gen_clears_resets(&attrs.clears, all);
            Some(quote! { ::ecmd::Parsed::Bool(#ch) => { #id = true; #resets } })
        }
        FieldRole::PolarityFlag(attrs) => {
            let ch = cf.id;
            let resets = gen_clears_resets(&attrs.clears, all);
            Some(quote! { ::ecmd::Parsed::Polar(#ch, p) => { #id = *p; #resets } })
        }
        FieldRole::ValuedFlag(attrs) => {
            let ch = cf.id;
            let resets = gen_clears_resets(&attrs.clears, all);
            let assign = gen_value_assign(id, cf.field, ch);
            Some(quote! { ::ecmd::Parsed::Value(#ch, v) => { #assign #resets } })
        }
        FieldRole::RepeatableValueFlag(attrs) => {
            let ch = cf.id;
            let resets = gen_clears_resets(&attrs.clears, all);
            let push = gen_repeatable_push(id, cf.field, ch);
            Some(quote! { ::ecmd::Parsed::Value(#ch, v) => { #push #resets } })
        }
        FieldRole::PolarValueFlag(attrs) => {
            let ch = cf.id;
            let resets = gen_clears_resets(&attrs.clears, all);
            Some(quote! {
                ::ecmd::Parsed::PolarValue(#ch, p, v) => {
                    #id.push(::ecmd::PolarVal::new(*p, (*v).to_owned()));
                    #resets
                }
            })
        }
        _ => None,
    }
}

pub fn gen_clears_resets(targets: &[Ident], all: &[ClassifiedField<'_>]) -> TokenStream {
    let stmts: Vec<_> = targets
        .iter()
        .filter_map(|target| {
            let cf = all.iter().find(|f| f.ident == target)?;
            let id = cf.ident;
            let reset = match &cf.role {
                FieldRole::BoolFlag(_) => quote! { #id = false; },
                FieldRole::PolarityFlag(_) => quote! { #id = ::ecmd::Polarity::Unset; },
                FieldRole::ValuedFlag(_) => quote! { #id = None; },
                FieldRole::RepeatableValueFlag(_) | FieldRole::PolarValueFlag(_) => {
                    quote! { #id = Vec::new(); }
                }
                _ => return None,
            };
            Some(reset)
        })
        .collect();
    quote! { #(#stmts)* }
}

pub fn gen_positionals(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let checks = gen_required_positional_checks(fields);
    let mut stmts = Vec::new();
    let mut idx = 0_usize;

    for cf in fields {
        let id = cf.ident;
        match &cf.role {
            FieldRole::OptionalPositional => {
                stmts.push(
                    quote! { #id = scan.operands().get(#idx).map(|value| (*value).to_owned()); },
                );
                idx = idx.saturating_add(1);
            }
            FieldRole::RequiredPositional => {
                stmts.push(quote! {
                    #id = scan.operands().get(#idx).map_or_else(String::new, |value| (*value).to_owned());
                });
                idx = idx.saturating_add(1);
            }
            FieldRole::Rest => {
                stmts.push(quote! {
                    #id = ::ecmd::Operands::from_args(
                        scan.operands().get(#idx..).unwrap_or_default()
                    );
                });
            }
            _ => {}
        }
    }
    quote! { #checks #(#stmts)* }
}

/// Collect every missing required positional first so the error names all of them
pub fn gen_required_positional_checks(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let mut checks = Vec::new();
    let mut idx = 0_usize;

    for cf in fields {
        let name = cf.ident.to_string();
        match &cf.role {
            FieldRole::OptionalPositional => idx = idx.saturating_add(1),
            FieldRole::RequiredPositional => {
                checks.push(quote! {
                    if scan.operands().get(#idx).is_none() {
                        __missing_required.push(#name.to_owned());
                    }
                });
                idx = idx.saturating_add(1);
            }
            _ => {}
        }
    }
    if checks.is_empty() {
        return TokenStream::new();
    }
    quote! {
        let mut __missing_required: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
        #(#checks)*
        if !__missing_required.is_empty() {
            return Err(::ecmd::Error::MissingRequired(__missing_required));
        }
    }
}

pub fn gen_value_assign(id: &Ident, field: &Field, ch: char) -> TokenStream {
    let inner = crate::classify::inner_type_name(&field.ty);
    if inner.as_deref() == Some("String") {
        quote! { #id = Some((*v).to_owned()); }
    } else {
        gen_parse_into(quote! { #id = Some }, ch)
    }
}

pub fn gen_repeatable_push(id: &Ident, field: &Field, ch: char) -> TokenStream {
    let inner = crate::classify::inner_type_name(&field.ty);
    if inner.as_deref() == Some("String") {
        quote! { #id.push((*v).to_owned()); }
    } else {
        gen_parse_into(quote! { #id.push }, ch)
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "quote! interpolation requires owned TokenStream"
)]
pub fn gen_parse_into(target: TokenStream, ch: char) -> TokenStream {
    quote! {
        #target(v.parse().map_err(|e| {
            ::ecmd::Error::InvalidValue {
                flag: format!("-{}", #ch),
                value: (*v).to_owned(),
                reason: format!("{e}"),
            }
        })?);
    }
}
