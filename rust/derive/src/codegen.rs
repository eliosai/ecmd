//! Code generation for the `Command` derive.

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Data, Fields, Field, Ident};

use crate::attrs::{CommandAttrs, FlagAttrs, extract_doc_comment, extract_doc_sections};
use crate::classify::{FieldRole, classify_field, field_ident};

/// Classified field with its role pre-computed.
struct ClassifiedField<'a> {
    field: &'a Field,
    ident: &'a Ident,
    role: FieldRole,
    desc: String,
    /// Identity char: the short flag, or a private-use codepoint for long-only.
    id: char,
}

/// Main expansion entry point.
pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let cmd = CommandAttrs::from_ast(&input.attrs)?;
    let sections = extract_doc_sections(&input.attrs);
    let raw_fields = extract_named_fields(input)?;
    let fields = classify_all(raw_fields)?;
    validate(&fields)?;
    let name = &input.ident;

    let meta_body = gen_meta(&cmd, &sections, &fields);
    let parse_body = gen_parse(&cmd, &fields);

    Ok(quote! {
        impl ::ecmd::meta::Command for #name {
            fn def() -> &'static ::ecmd::meta::CommandDef {
                static DEF: ::ecmd::meta::CommandDef = #meta_body;
                &DEF
            }

            fn parse(args: &[&str]) -> ::core::result::Result<Self, ::ecmd::error::Error> {
                #parse_body
            }
        }
    })
}

// ── Extraction + classification ─────────────────────────────────

fn extract_named_fields(input: &DeriveInput) -> syn::Result<&syn::punctuated::Punctuated<Field, syn::token::Comma>> {
    match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => Ok(&n.named),
            _ => Err(syn::Error::new_spanned(input, "only named structs")),
        },
        _ => Err(syn::Error::new_spanned(input, "only structs")),
    }
}

fn classify_all(fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>) -> syn::Result<Vec<ClassifiedField<'_>>> {
    fields.iter().enumerate().map(|(i, f)| {
        let role = classify_field(f)?;
        let desc = extract_doc_comment(&f.attrs);
        let id = field_id(&role, i);
        Ok(ClassifiedField { field: f, ident: field_ident(f), role, desc, id })
    }).collect()
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

// ── Validation ──────────────────────────────────────────────────

fn validate(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
    check_duplicate_flags(fields)?;
    check_operands_last(fields)?;
    check_positional_ordering(fields)?;
    check_clears_targets(fields)
}

fn check_positional_ordering(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
    let mut saw_optional = false;
    for cf in fields {
        match &cf.role {
            FieldRole::OptionalPositional => saw_optional = true,
            FieldRole::RequiredPositional if saw_optional => {
                return Err(syn::Error::new_spanned(
                    cf.field, "required positional cannot follow optional positional",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_duplicate_flags(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
    let mut seen = HashSet::new();
    for cf in fields {
        if cf.id != '\0' && !seen.insert(cf.id) {
            return Err(syn::Error::new_spanned(
                cf.field, format!("duplicate flag '{}'", cf.id),
            ));
        }
    }
    Ok(())
}

fn check_operands_last(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
    let mut saw_rest = false;
    for cf in fields {
        if saw_rest {
            return Err(syn::Error::new_spanned(
                cf.field, "no fields allowed after Operands",
            ));
        }
        if matches!(cf.role, FieldRole::Rest) {
            saw_rest = true;
        }
    }
    Ok(())
}

fn check_clears_targets(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
    let flag_names: HashSet<&Ident> = fields.iter()
        .filter(|cf| flag_char(&cf.role).is_some())
        .map(|cf| cf.ident)
        .collect();

    for cf in fields {
        for target in clears_targets(&cf.role) {
            if target == cf.ident {
                return Err(syn::Error::new_spanned(
                    target, "flag cannot clear itself",
                ));
            }
            if !flag_names.contains(target) {
                return Err(syn::Error::new_spanned(
                    target, format!("`{target}` is not a flag field"),
                ));
            }
        }
    }
    Ok(())
}

// ── Parse codegen ───────────────────────────────────────────────

fn gen_parse(cmd: &CommandAttrs, fields: &[ClassifiedField<'_>]) -> TokenStream {
    let on_unknown = if cmd.lenient {
        quote! { ::ecmd::parse::OnUnknown::PassThrough }
    } else {
        quote! { ::ecmd::parse::OnUnknown::Reject }
    };

    let flag_defs = gen_flag_defs(cmd, fields);
    let inits = gen_inits(fields);
    let dispatch = gen_dispatch(fields);
    let positionals = gen_positionals(fields);
    let names: Vec<_> = fields.iter().map(|cf| cf.ident).collect();
    let style = style_tokens(cmd);
    let permute = !cmd.no_permute;

    quote! {
        static FLAGS: &[::ecmd::parse::FlagDef] = &[#flag_defs];
        let result = ::ecmd::parse::scan(args, FLAGS, #on_unknown, #style, #permute)?;

        #inits

        for flag in &result.flags {
            match flag {
                #dispatch
                _ => {}
            }
        }

        #positionals

        Ok(Self { #(#names),* })
    }
}

fn gen_flag_defs(cmd: &CommandAttrs, fields: &[ClassifiedField<'_>]) -> TokenStream {
    let mut defs: Vec<TokenStream> = Vec::new();

    for cf in fields {
        if let Some(literal) = flag_def_literal(cf, fields, cmd) {
            defs.push(literal);
        }
    }

    for ch in cmd.noop.chars() {
        defs.push(quote! {
            ::ecmd::parse::FlagDef { ch: #ch, long: "", aliases: &[], kind: ::ecmd::parse::FlagKind::Noop, clears: &[], desc: "", value_name: "", hidden: false }
        });
    }

    quote! { #(#defs),* }
}

fn gen_inits(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let stmts: Vec<_> = fields.iter().map(|cf| {
        let id = cf.ident;
        match &cf.role {
            FieldRole::BoolFlag(_) => quote! { let mut #id = false; },
            FieldRole::PolarityFlag(_) => quote! { let mut #id = ::ecmd::polarity::Polarity::Unset; },
            FieldRole::ValuedFlag(_)
            | FieldRole::OptionalPositional => quote! { let mut #id = None; },
            FieldRole::PolarValueFlag(_)
            | FieldRole::RepeatableValueFlag(_) => quote! { let mut #id = Vec::new(); },
            FieldRole::RequiredPositional | FieldRole::Rest => quote! { let #id; },
        }
    }).collect();
    quote! { #(#stmts)* }
}

fn gen_dispatch(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let arms: Vec<_> = fields.iter().filter_map(|cf| {
        gen_single_dispatch(cf, fields)
    }).collect();
    quote! { #(#arms)* }
}

fn gen_single_dispatch(cf: &ClassifiedField<'_>, all: &[ClassifiedField<'_>]) -> Option<TokenStream> {
    let id = cf.ident;
    match &cf.role {
        FieldRole::BoolFlag(attrs) => {
            let ch = cf.id;
            let resets = gen_clears_resets(&attrs.clears, all);
            Some(quote! { ::ecmd::parse::Parsed::Bool(#ch) => { #id = true; #resets } })
        }
        FieldRole::PolarityFlag(attrs) => {
            let ch = cf.id;
            let resets = gen_clears_resets(&attrs.clears, all);
            Some(quote! { ::ecmd::parse::Parsed::Polar(#ch, p) => { #id = *p; #resets } })
        }
        FieldRole::ValuedFlag(attrs) => {
            let ch = cf.id;
            let resets = gen_clears_resets(&attrs.clears, all);
            let assign = gen_value_assign(id, cf.field, ch);
            Some(quote! { ::ecmd::parse::Parsed::Value(#ch, v) => { #assign #resets } })
        }
        FieldRole::RepeatableValueFlag(attrs) => {
            let ch = cf.id;
            let resets = gen_clears_resets(&attrs.clears, all);
            let push = gen_repeatable_push(id, cf.field, ch);
            Some(quote! { ::ecmd::parse::Parsed::Value(#ch, v) => { #push #resets } })
        }
        FieldRole::PolarValueFlag(attrs) => {
            let ch = cf.id;
            let resets = gen_clears_resets(&attrs.clears, all);
            Some(quote! {
                ::ecmd::parse::Parsed::PolarValue(#ch, p, v) => {
                    #id.push(::ecmd::polarity::PolarVal::new(*p, v.clone()));
                    #resets
                }
            })
        }
        _ => None,
    }
}

fn gen_clears_resets(targets: &[Ident], all: &[ClassifiedField<'_>]) -> TokenStream {
    let stmts: Vec<_> = targets.iter().filter_map(|target| {
        let cf = all.iter().find(|f| f.ident == target)?;
        let id = cf.ident;
        let reset = match &cf.role {
            FieldRole::BoolFlag(_) => quote! { #id = false; },
            FieldRole::PolarityFlag(_) => quote! { #id = ::ecmd::polarity::Polarity::Unset; },
            FieldRole::ValuedFlag(_) => quote! { #id = None; },
            FieldRole::RepeatableValueFlag(_)
            | FieldRole::PolarValueFlag(_) => quote! { #id = Vec::new(); },
            _ => return None,
        };
        Some(reset)
    }).collect();
    quote! { #(#stmts)* }
}

fn gen_positionals(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let mut stmts = Vec::new();
    let mut idx = 0_usize;

    for cf in fields {
        let id = cf.ident;
        let name = id.to_string();
        match &cf.role {
            FieldRole::OptionalPositional => {
                stmts.push(quote! { #id = result.operands.get(#idx).cloned(); });
                idx = idx.saturating_add(1);
            }
            FieldRole::RequiredPositional => {
                stmts.push(quote! {
                    #id = result.operands.get(#idx).cloned()
                        .ok_or_else(|| ::ecmd::error::Error::MissingRequired(#name.to_owned()))?;
                });
                idx = idx.saturating_add(1);
            }
            FieldRole::Rest => {
                stmts.push(quote! {
                    #id = ::ecmd::operands::Operands::from_args(
                        result.operands.get(#idx..).unwrap_or_default()
                    );
                });
            }
            _ => {}
        }
    }
    quote! { #(#stmts)* }
}

// ── Meta codegen ────────────────────────────────────────────────

fn gen_meta(
    cmd: &CommandAttrs,
    sections: &crate::attrs::DocSections,
    fields: &[ClassifiedField<'_>],
) -> TokenStream {
    let name = &cmd.name;
    let about = &sections.about;
    let short_doc = &cmd.short_doc;
    let style = style_tokens(cmd);
    let on_unknown = if cmd.lenient {
        quote! { ::ecmd::parse::OnUnknown::PassThrough }
    } else {
        quote! { ::ecmd::parse::OnUnknown::Reject }
    };
    let flag_metas = gen_flag_metas(cmd, fields);
    let pos_metas = gen_positional_metas(fields);
    let has_rest = fields.iter().any(|cf| matches!(cf.role, FieldRole::Rest));
    let tag_keys: Vec<&str> = cmd.tags.iter().map(|(k, _)| k.as_str()).collect();
    let tag_vals: Vec<&str> = cmd.tags.iter().map(|(_, v)| v.as_str()).collect();

    let desc_lines: &Vec<String> = &sections.description;
    let extra_lines: &Vec<String> = if cmd.extra_help.is_empty() {
        &sections.extra
    } else {
        &cmd.extra_help
    };
    let exit_lines: &Vec<String> = &sections.exit_status;

    quote! {
        ::ecmd::meta::CommandDef {
            name: #name,
            about: #about,
            short_doc: #short_doc,
            style: #style,
            on_unknown: #on_unknown,
            flags: &[#flag_metas],
            positionals: &[#pos_metas],
            has_rest: #has_rest,
            tags: &[#( (#tag_keys, #tag_vals) ),*],
            description: &[#( #desc_lines ),*],
            extra: &[#( #extra_lines ),*],
            exit_status: &[#( #exit_lines ),*],
        }
    }
}

fn gen_flag_metas(cmd: &CommandAttrs, fields: &[ClassifiedField<'_>]) -> TokenStream {
    let defs: Vec<_> = fields.iter().filter_map(|cf| flag_def_literal(cf, fields, cmd)).collect();
    quote! { #(#defs),* }
}

/// One `FlagDef { … }` literal for a flag field, shared by parse and meta codegen.
fn flag_def_literal(cf: &ClassifiedField<'_>, fields: &[ClassifiedField<'_>], cmd: &CommandAttrs) -> Option<TokenStream> {
    let (_, kind) = flag_def_tokens(&cf.role)?;
    let ch = cf.id;
    let clears: Vec<char> = resolve_clears(&cf.role, fields);
    let desc = &cf.desc;
    let value_name = flag_value_name(&cf.role);
    let long = flag_long(cf, cmd);
    let aliases = flag_aliases(&cf.role);
    let hidden = flag_attrs(&cf.role).is_some_and(|a| a.hidden);
    Some(quote! {
        ::ecmd::parse::FlagDef { ch: #ch, long: #long, aliases: &[#(#aliases),*], kind: #kind, clears: &[#(#clears),*], desc: #desc, value_name: #value_name, hidden: #hidden }
    })
}

/// Alias long names declared via `#[flag(alias = "…")]`.
fn flag_aliases(role: &FieldRole) -> Vec<String> {
    flag_attrs(role).map(|a| a.aliases.clone()).unwrap_or_default()
}

/// GNU long name for a flag: explicit `long=`, else kebab field name in gnu style, else empty.
fn flag_long(cf: &ClassifiedField<'_>, cmd: &CommandAttrs) -> String {
    if let Some(attrs) = flag_attrs(&cf.role)
        && let Some(explicit) = &attrs.long
    {
        return explicit.clone();
    }
    if cmd.style == "gnu" {
        return kebab(cf.ident);
    }
    String::new()
}

/// Convert a field ident to a kebab-case long name (`nchars_exact` → "nchars-exact").
fn kebab(ident: &Ident) -> String {
    ident.to_string().trim_matches('_').replace('_', "-")
}

/// The parsing style tokens for this command (Gnu when opted in, else Posix).
fn style_tokens(cmd: &CommandAttrs) -> TokenStream {
    if cmd.style == "gnu" {
        quote! { ::ecmd::style::Style::Gnu }
    } else {
        quote! { ::ecmd::style::Style::Posix }
    }
}

fn gen_positional_metas(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let defs: Vec<_> = fields.iter().filter_map(|cf| {
        let required = match &cf.role {
            FieldRole::RequiredPositional => true,
            FieldRole::OptionalPositional => false,
            _ => return None,
        };
        let name = cf.ident.to_string();
        let desc = &cf.desc;
        Some(quote! {
            ::ecmd::meta::PositionalDef { name: #name, required: #required, desc: #desc }
        })
    }).collect();
    quote! { #(#defs),* }
}

// ── Value assignment (FromStr) ───────────────────────────────────

fn gen_value_assign(id: &Ident, field: &Field, ch: char) -> TokenStream {
    let inner = crate::classify::inner_type_name(&field.ty);
    if inner.as_deref() == Some("String") {
        quote! { #id = Some(v.clone()); }
    } else {
        gen_parse_into(quote! { #id = Some }, ch)
    }
}

fn gen_repeatable_push(id: &Ident, field: &Field, ch: char) -> TokenStream {
    let inner = crate::classify::inner_type_name(&field.ty);
    if inner.as_deref() == Some("String") {
        quote! { #id.push(v.clone()); }
    } else {
        gen_parse_into(quote! { #id.push }, ch)
    }
}

#[expect(clippy::needless_pass_by_value, reason = "quote! interpolation requires owned TokenStream")]
fn gen_parse_into(target: TokenStream, ch: char) -> TokenStream {
    quote! {
        #target(v.parse().map_err(|e| {
            ::ecmd::error::Error::InvalidValue {
                flag: format!("-{}", #ch),
                value: v.clone(),
                reason: format!("{e}"),
            }
        })?);
    }
}

// ── Helpers ─────────────────────────────────────────────────────

const fn flag_char(role: &FieldRole) -> Option<char> {
    match role {
        FieldRole::BoolFlag(a)
        | FieldRole::PolarityFlag(a)
        | FieldRole::ValuedFlag(a)
        | FieldRole::RepeatableValueFlag(a)
        | FieldRole::PolarValueFlag(a) => Some(a.short),
        _ => None,
    }
}

const fn flag_attrs(role: &FieldRole) -> Option<&FlagAttrs> {
    match role {
        FieldRole::BoolFlag(a)
        | FieldRole::PolarityFlag(a)
        | FieldRole::ValuedFlag(a)
        | FieldRole::RepeatableValueFlag(a)
        | FieldRole::PolarValueFlag(a) => Some(a),
        _ => None,
    }
}

fn clears_targets(role: &FieldRole) -> &[Ident] {
    match role {
        FieldRole::BoolFlag(a)
        | FieldRole::PolarityFlag(a)
        | FieldRole::ValuedFlag(a)
        | FieldRole::RepeatableValueFlag(a)
        | FieldRole::PolarValueFlag(a) => &a.clears,
        _ => &[],
    }
}

fn flag_def_tokens(role: &FieldRole) -> Option<(char, TokenStream)> {
    match role {
        FieldRole::BoolFlag(a) => Some((a.short, quote! { ::ecmd::parse::FlagKind::Bool })),
        FieldRole::PolarityFlag(a) => Some((a.short, quote! { ::ecmd::parse::FlagKind::Polar })),
        FieldRole::ValuedFlag(a) | FieldRole::RepeatableValueFlag(a) => {
            Some((a.short, quote! { ::ecmd::parse::FlagKind::Value }))
        }
        FieldRole::PolarValueFlag(a) => Some((a.short, quote! { ::ecmd::parse::FlagKind::PolarValue })),
        _ => None,
    }
}

fn resolve_clears(role: &FieldRole, all: &[ClassifiedField<'_>]) -> Vec<char> {
    clears_targets(role).iter().filter_map(|target| {
        all.iter().find(|cf| cf.ident == target).map(|cf| cf.id)
    }).collect()
}

fn flag_value_name(role: &FieldRole) -> &str {
    match role {
        FieldRole::ValuedFlag(a)
        | FieldRole::RepeatableValueFlag(a)
        | FieldRole::PolarValueFlag(a) => {
            if a.value_name.is_empty() { "ARG" } else { &a.value_name }
        }
        _ => "",
    }
}
