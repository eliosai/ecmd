//! Code generation for the `Command` derive.

use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, Ident};

use crate::attrs::{
    CommandAttrs, FlagAttrs, OperandAttrs, RepeatAttr, extract_doc_comment, extract_doc_sections,
};
use crate::classify::{FieldRole, classify_field, field_ident};

/// Classified field with its role pre-computed.
struct ClassifiedField<'a> {
    field: &'a Field,
    ident: &'a Ident,
    role: FieldRole,
    desc: String,
    /// Identity char: the short flag, or a private-use codepoint for long-only.
    id: char,
    /// Help presentation for a positional or rest field.
    operand: OperandAttrs,
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

// ── Extraction + classification ─────────────────────────────────

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
                    cf.field,
                    "required positional cannot follow optional positional",
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
                cf.field,
                format!("duplicate flag '{}'", cf.id),
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
                cf.field,
                "no fields allowed after Operands",
            ));
        }
        if matches!(cf.role, FieldRole::Rest) {
            saw_rest = true;
        }
    }
    Ok(())
}

fn check_clears_targets(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
    let flag_names: HashSet<&Ident> = fields
        .iter()
        .filter(|cf| flag_char(&cf.role).is_some())
        .map(|cf| cf.ident)
        .collect();

    for cf in fields {
        for target in clears_targets(&cf.role) {
            if target == cf.ident {
                return Err(syn::Error::new_spanned(target, "flag cannot clear itself"));
            }
            if !flag_names.contains(target) {
                return Err(syn::Error::new_spanned(
                    target,
                    format!("`{target}` is not a flag field"),
                ));
            }
        }
    }
    Ok(())
}

// ── Parse codegen ───────────────────────────────────────────────

fn gen_parse(fields: &[ClassifiedField<'_>]) -> TokenStream {
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

fn gen_inits(fields: &[ClassifiedField<'_>]) -> TokenStream {
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

fn gen_dispatch(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let arms: Vec<_> = fields
        .iter()
        .filter_map(|cf| gen_single_dispatch(cf, fields))
        .collect();
    quote! { #(#arms)* }
}

fn gen_single_dispatch(
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

fn gen_clears_resets(targets: &[Ident], all: &[ClassifiedField<'_>]) -> TokenStream {
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

fn gen_positionals(fields: &[ClassifiedField<'_>]) -> TokenStream {
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
fn gen_required_positional_checks(fields: &[ClassifiedField<'_>]) -> TokenStream {
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

// ── Parse policy codegen ────────────────────────────────────────

struct ValuePolicyLiteral {
    ch: char,
    mode: TokenStream,
    default: String,
}

fn gen_policy(cmd: &CommandAttrs, fields: &[ClassifiedField<'_>]) -> syn::Result<TokenStream> {
    let mut values = Vec::new();
    let mut numeric_operands = Vec::new();
    let mut first_numeric = None;
    let mut equals_only = Vec::new();
    let mut attached_values = Vec::new();
    let mut separated_values = Vec::new();
    let mut prefixed_values = Vec::new();
    let mut exclusive_groups = Vec::new();
    let mut seen_values = HashSet::new();
    let exact_long = cmd.tags.iter().any(|(key, _)| key == "exact_long");
    for (key, value) in &cmd.tags {
        let mode = match key.as_str() {
            "optional_values" => Some(quote! { ::ecmd::__private::ValueMode::AttachedOrDefault }),
            "optional_next_values" => Some(quote! { ::ecmd::__private::ValueMode::NextOrDefault }),
            "numeric_next_values" => {
                Some(quote! { ::ecmd::__private::ValueMode::NumericNextOrDefault })
            }
            "optional_numeric_next_values" => {
                Some(quote! { ::ecmd::__private::ValueMode::OptionalNumericNextOrDefault })
            }
            "exact_short_defaults" => {
                Some(quote! { ::ecmd::__private::ValueMode::ExactShortDefault })
            }
            "optional_any_next_values" => {
                Some(quote! { ::ecmd::__private::ValueMode::AnyNextOrDefault })
            }
            _ => None,
        };
        if let Some(mode) = mode {
            append_value_rules(value, &mode, fields, &mut values, &mut seen_values)?;
        } else if key == "numeric_operands" {
            append_numeric_operands(value, fields, &mut numeric_operands)?;
        } else if key == "first_numeric_value" {
            first_numeric = Some(policy_field(fields, value)?.id);
        } else if key == "equals_only" {
            append_flag_identities(value, fields, &mut equals_only)?;
        } else if key == "attached_values" {
            append_flag_identities(value, fields, &mut attached_values)?;
        } else if key == "separated_values" {
            append_flag_identities(value, fields, &mut separated_values)?;
        } else if key == "prefixed_values" {
            append_flag_identities(value, fields, &mut prefixed_values)?;
        } else if key == "exclusive_flags" {
            append_exclusive_groups(value, fields, &mut exclusive_groups)?;
        }
    }
    Ok(policy_literal(&PolicySpec {
        values: &values,
        no_implicit_version: cmd.no_implicit_version,
        numeric_operands: &numeric_operands,
        first_numeric,
        exact_long,
        equals_only: &equals_only,
        attached_values: &attached_values,
        separated_values: &separated_values,
        prefixed_values: &prefixed_values,
        exclusive_groups: &exclusive_groups,
    }))
}

fn append_flag_identities(
    raw: &str,
    fields: &[ClassifiedField<'_>],
    identities: &mut Vec<char>,
) -> syn::Result<()> {
    for name in raw.split(',').filter(|name| !name.is_empty()) {
        let identity = policy_field(fields, name.trim())?.id;
        if !identities.contains(&identity) {
            identities.push(identity);
        }
    }
    Ok(())
}

fn append_exclusive_groups(
    raw: &str,
    fields: &[ClassifiedField<'_>],
    rules: &mut Vec<(char, u16)>,
) -> syn::Result<()> {
    let offset = rules
        .iter()
        .map(|(_, group)| *group)
        .max()
        .map_or(0, |group| group.saturating_add(1));
    for (group, alternatives) in raw.split(';').enumerate() {
        let group = u16::try_from(group)
            .map_err(|_error| syn::Error::new(Span::call_site(), "too many exclusive groups"))?;
        let group = offset
            .checked_add(group)
            .ok_or_else(|| syn::Error::new(Span::call_site(), "too many exclusive groups"))?;
        for name in alternatives.split(',').filter(|name| !name.is_empty()) {
            rules.push((policy_any_field(fields, name.trim())?.id, group));
        }
    }
    Ok(())
}

fn append_value_rules(
    raw: &str,
    mode: &TokenStream,
    fields: &[ClassifiedField<'_>],
    values: &mut Vec<ValuePolicyLiteral>,
    seen: &mut HashSet<char>,
) -> syn::Result<()> {
    for entry in raw.split(',').filter(|entry| !entry.is_empty()) {
        let (name, default) = entry.split_once('=').unwrap_or((entry, ""));
        let field = policy_field(fields, name.trim())?;
        if !seen.insert(field.id) {
            return Err(syn::Error::new_spanned(
                field.field,
                "duplicate value policy",
            ));
        }
        values.push(ValuePolicyLiteral {
            ch: field.id,
            mode: mode.clone(),
            default: default.to_owned(),
        });
    }
    Ok(())
}

fn append_numeric_operands(
    raw: &str,
    fields: &[ClassifiedField<'_>],
    rules: &mut Vec<(char, char)>,
) -> syn::Result<()> {
    for entry in raw.split(',').filter(|entry| !entry.is_empty()) {
        let Some((name, prefix)) = entry.split_once('=') else {
            return Err(syn::Error::new(
                Span::call_site(),
                "numeric operand policy needs field=prefix",
            ));
        };
        let field = policy_field(fields, name.trim())?;
        let mut chars = prefix.chars();
        let Some(prefix) = chars.next().filter(|_| chars.next().is_none()) else {
            return Err(syn::Error::new_spanned(
                field.field,
                "numeric operand prefix must be one character",
            ));
        };
        rules.push((field.id, prefix));
    }
    Ok(())
}

fn policy_field<'a, 'field>(
    fields: &'a [ClassifiedField<'field>],
    name: &str,
) -> syn::Result<&'a ClassifiedField<'field>> {
    let field = policy_any_field(fields, name)?;
    if matches!(
        field.role,
        FieldRole::ValuedFlag(_) | FieldRole::RepeatableValueFlag(_) | FieldRole::PolarValueFlag(_)
    ) {
        Ok(field)
    } else {
        Err(syn::Error::new_spanned(
            field.field,
            "policy field must be a valued flag",
        ))
    }
}

fn policy_any_field<'a, 'field>(
    fields: &'a [ClassifiedField<'field>],
    name: &str,
) -> syn::Result<&'a ClassifiedField<'field>> {
    let field = if let Some(field) = fields.iter().find(|field| field.ident == name) {
        field
    } else {
        unique_policy_long(fields, name)?
    };
    if flag_attrs(&field.role).is_some() {
        Ok(field)
    } else {
        Err(syn::Error::new_spanned(
            field.field,
            "policy field must be a flag",
        ))
    }
}

fn unique_policy_long<'a, 'field>(
    fields: &'a [ClassifiedField<'field>],
    name: &str,
) -> syn::Result<&'a ClassifiedField<'field>> {
    let mut matches = fields.iter().filter(|field| policy_long(field, name));
    let Some(field) = matches.next() else {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("unknown policy field `{name}`"),
        ));
    };
    if matches.next().is_some() {
        return Err(syn::Error::new_spanned(
            field.field,
            "ambiguous policy field",
        ));
    }
    Ok(field)
}

fn policy_long(field: &ClassifiedField<'_>, name: &str) -> bool {
    flag_attrs(&field.role).is_some_and(|attrs| {
        attrs.long.as_deref() == Some(name) || (attrs.long.is_none() && kebab(field.ident) == name)
    })
}

struct PolicySpec<'a> {
    values: &'a [ValuePolicyLiteral],
    no_implicit_version: bool,
    numeric_operands: &'a [(char, char)],
    first_numeric: Option<char>,
    exact_long: bool,
    equals_only: &'a [char],
    attached_values: &'a [char],
    separated_values: &'a [char],
    prefixed_values: &'a [char],
    exclusive_groups: &'a [(char, u16)],
}

/// The `POLICY` static and the rule arrays it borrows
fn policy_literal(spec: &PolicySpec<'_>) -> TokenStream {
    let value_rules = spec.values.iter().map(|rule| {
        let ch = rule.ch;
        let mode = &rule.mode;
        let default = &rule.default;
        quote! { ::ecmd::__private::ValueRule { ch: #ch, mode: #mode, default: ::std::borrow::Cow::Borrowed(#default) } }
    });
    let value_count = spec.values.len();
    let numeric_rules = spec.numeric_operands.iter().map(|(ch, prefix)| {
        quote! { ::ecmd::__private::NumericRule { ch: #ch, prefix: #prefix } }
    });
    let numeric_count = spec.numeric_operands.len();
    let first = spec
        .first_numeric
        .map_or_else(|| quote! { None }, |ch| quote! { Some(#ch) });
    let exclusive_rules = spec.exclusive_groups.iter().map(|(ch, group)| {
        quote! { ::ecmd::__private::ExclusiveRule { ch: #ch, group: #group } }
    });
    let exclusive_count = spec.exclusive_groups.len();
    let exact_long = spec.exact_long;
    let no_implicit_version = spec.no_implicit_version;
    let equals_only = identities_literal(spec.equals_only);
    let attached_values = identities_literal(spec.attached_values);
    let separated_values = identities_literal(spec.separated_values);
    let prefixed_values = identities_literal(spec.prefixed_values);
    quote! {
        static VALUE_RULES: [::ecmd::__private::ValueRule; #value_count] = [#(#value_rules),*];
        static NUMERIC_RULES: [::ecmd::__private::NumericRule; #numeric_count] = [#(#numeric_rules),*];
        static EXCLUSIVE_RULES: [::ecmd::__private::ExclusiveRule; #exclusive_count] = [#(#exclusive_rules),*];
        static POLICY: ::ecmd::__private::Policy = ::ecmd::__private::Policy {
            value_rules: ::std::borrow::Cow::Borrowed(&VALUE_RULES),
            numeric_operands: ::std::borrow::Cow::Borrowed(&NUMERIC_RULES),
            first_numeric_value: #first,
            exact_long: #exact_long,
            no_implicit_version: #no_implicit_version,
            equals_only: ::std::borrow::Cow::Borrowed(#equals_only),
            attached_values: ::std::borrow::Cow::Borrowed(#attached_values),
            separated_values: ::std::borrow::Cow::Borrowed(#separated_values),
            prefixed_values: ::std::borrow::Cow::Borrowed(#prefixed_values),
            exclusive_groups: ::std::borrow::Cow::Borrowed(&EXCLUSIVE_RULES),
        };
    }
}

fn identities_literal(identities: &[char]) -> TokenStream {
    quote! { &[#(#identities),*] }
}

// ── Meta codegen ────────────────────────────────────────────────

/// The statics one definition is built from, ending in a borrow of the definition itself
fn gen_meta(
    cmd: &CommandAttrs,
    sections: &crate::attrs::DocSections,
    fields: &[ClassifiedField<'_>],
    policy: &TokenStream,
) -> TokenStream {
    let name = &cmd.name;
    let about = &sections.about;
    let short_doc = &cmd.short_doc;
    let style = style_tokens(cmd);
    let help_style = help_style_tokens(cmd);
    let lenient = cmd.lenient;
    let permute = !cmd.no_permute;
    let flags = gen_flag_metas(cmd, fields);
    let positionals = gen_positional_metas(fields);
    let (rest_static, rest) = gen_rest(fields);
    let tags = gen_tags(cmd);
    let extra_lines: &Vec<String> = if cmd.extra_help.is_empty() {
        &sections.extra
    } else {
        &cmd.extra_help
    };
    let description = texts_static(&quote! { DESCRIPTION }, &sections.description);
    let extra = texts_static(&quote! { EXTRA }, extra_lines);
    let exit_status = texts_static(&quote! { EXIT_STATUS }, &sections.exit_status);
    quote! {
        #flags
        #positionals
        #rest_static
        #policy
        #tags
        #description
        #extra
        #exit_status
        static DEF: ::ecmd::Def = ::ecmd::Def::from_raw(::ecmd::__private::RawDef {
            name: #name,
            about: #about,
            short_doc: #short_doc,
            style: #style,
            help_style: #help_style,
            lenient: #lenient,
            permute: #permute,
            flags: &FLAGS,
            positionals: &POSITIONALS,
            rest: #rest,
            policy: &POLICY,
            tags: &TAGS,
            description: &DESCRIPTION,
            extra: &EXTRA,
            exit_status: &EXIT_STATUS,
        });
        &DEF
    }
}

/// A static array of borrowed text lines under `name`
fn texts_static(name: &TokenStream, lines: &[String]) -> TokenStream {
    let count = lines.len();
    quote! {
        static #name: [::std::borrow::Cow<'static, str>; #count] =
            [#(::std::borrow::Cow::Borrowed(#lines)),*];
    }
}

/// A static text array for a flag list, or a bare empty slice when there is nothing to hold
fn texts_ref(name: &str, lines: &[String]) -> (TokenStream, TokenStream) {
    if lines.is_empty() {
        return (TokenStream::new(), quote! { &[] });
    }
    let ident = Ident::new(name, Span::call_site());
    (texts_static(&quote! { #ident }, lines), quote! { &#ident })
}

fn gen_tags(cmd: &CommandAttrs) -> TokenStream {
    let count = cmd.tags.len();
    let keys = cmd.tags.iter().map(|(key, _)| key);
    let values = cmd.tags.iter().map(|(_, value)| value);
    quote! {
        static TAGS: [::ecmd::__private::Tag; #count] =
            [#(::ecmd::__private::Tag::new_static(#keys, #values)),*];
    }
}

/// The rest slot as a static, and the borrow the definition takes of it
fn gen_rest(fields: &[ClassifiedField<'_>]) -> (TokenStream, TokenStream) {
    let Some(cf) = fields.iter().find(|cf| matches!(cf.role, FieldRole::Rest)) else {
        return (TokenStream::new(), quote! { None });
    };
    let name = cf.ident.to_string();
    let desc = &cf.desc;
    let label = &cf.operand.label;
    let default_value = &cf.operand.default_value;
    let hidden = cf.operand.hidden;
    let required = cf.operand.required;
    let rest = quote! {
        static REST: ::ecmd::Positional = ::ecmd::Positional::from_raw(::ecmd::__private::RawPositional {
            name: #name,
            required: #required,
            desc: #desc,
            label: #label,
            default_value: #default_value,
            hidden: #hidden,
            spread: false,
        });
    };
    (rest, quote! { Some(&REST) })
}

/// The `FLAGS` static and the text statics its entries borrow
fn gen_flag_metas(cmd: &CommandAttrs, fields: &[ClassifiedField<'_>]) -> TokenStream {
    let mut statics = Vec::new();
    let mut defs: Vec<_> = fields
        .iter()
        .enumerate()
        .filter_map(|(index, cf)| flag_def_literal(cf, cmd, index, &mut statics))
        .collect();
    for ch in cmd.noop.chars() {
        defs.push(quote! {
            ::ecmd::Flag::from_raw(::ecmd::__private::RawFlag {
                id: #ch,
                kind: ::ecmd::FlagKind::Noop,
                ..::ecmd::__private::RawFlag::EMPTY
            })
        });
    }
    let count = defs.len();
    quote! {
        #(#statics)*
        static FLAGS: [::ecmd::Flag; #count] = [#(#defs),*];
    }
}

/// One `Flag::from_raw` literal for a flag field, its text lists pushed as statics
fn flag_def_literal(
    cf: &ClassifiedField<'_>,
    cmd: &CommandAttrs,
    index: usize,
    statics: &mut Vec<TokenStream>,
) -> Option<TokenStream> {
    let (_, kind) = flag_def_tokens(&cf.role)?;
    let attrs = flag_attrs(&cf.role);
    let id = cf.id;
    let desc = &cf.desc;
    let value_name = flag_value_name(&cf.role);
    let long = flag_long(cf, cmd);
    let hidden = attrs.is_some_and(|a| a.hidden);
    let implemented = attrs.is_none_or(|a| a.implemented);
    let repeatable = attrs.is_some_and(|attrs| attrs.repeat == RepeatAttr::Repeatable)
        || !cmd.no_override
        || matches!(cf.role, FieldRole::RepeatableValueFlag(_));
    let allow_hyphen_values = attrs.is_none_or(|attrs| attrs.allow_hyphen_values);
    let default_value = attrs
        .map(|attrs| attrs.default_value.clone())
        .unwrap_or_default();
    let help_label = attrs
        .map(|attrs| attrs.help_label.clone())
        .unwrap_or_default();
    let lists = [
        (
            "ALIASES",
            attrs.map(|a| a.aliases.clone()).unwrap_or_default(),
        ),
        (
            "VALUES",
            attrs.map(|a| a.possible_values.clone()).unwrap_or_default(),
        ),
        (
            "HELP_VALUES",
            attrs.map(|a| a.help_values.clone()).unwrap_or_default(),
        ),
        (
            "VISIBLE_ALIASES",
            attrs.map(|a| a.visible_aliases.clone()).unwrap_or_default(),
        ),
    ];
    let mut refs = Vec::new();
    for (suffix, lines) in &lists {
        let (text_static, reference) = texts_ref(&format!("FLAG_{index}_{suffix}"), lines);
        statics.push(text_static);
        refs.push(reference);
    }
    let [aliases, possible_values, help_values, visible_aliases] =
        <[TokenStream; 4]>::try_from(refs).ok()?;
    Some(quote! {
        ::ecmd::Flag::from_raw(::ecmd::__private::RawFlag {
            id: #id,
            long: #long,
            aliases: #aliases,
            kind: #kind,
            desc: #desc,
            value_name: #value_name,
            hidden: #hidden,
            implemented: #implemented,
            repeatable: #repeatable,
            allow_hyphen_values: #allow_hyphen_values,
            possible_values: #possible_values,
            help_values: #help_values,
            default_value: #default_value,
            help_label: #help_label,
            visible_aliases: #visible_aliases,
        })
    })
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
        quote! { ::ecmd::Style::Gnu }
    } else {
        quote! { ::ecmd::Style::Posix }
    }
}

/// The declared help dialect, defaulting to the one implied by the parse style.
fn help_style_tokens(cmd: &CommandAttrs) -> TokenStream {
    match cmd.help_style.as_deref() {
        Some("bash") => quote! { ::ecmd::HelpStyle::Bash },
        Some("gnu") => quote! { ::ecmd::HelpStyle::Gnu },
        Some("clap") => quote! { ::ecmd::HelpStyle::Clap },
        Some("clap_wide") => quote! { ::ecmd::HelpStyle::ClapWide },
        Some("util_linux") => quote! { ::ecmd::HelpStyle::UtilLinux },
        _ => {
            let style = style_tokens(cmd);
            quote! { ::ecmd::HelpStyle::from_parse_style(#style) }
        }
    }
}

fn gen_positional_metas(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let defs: Vec<_> = fields
        .iter()
        .filter_map(|cf| {
            let required = match &cf.role {
                FieldRole::RequiredPositional => true,
                FieldRole::OptionalPositional => false,
                _ => return None,
            };
            let name = cf.ident.to_string();
            let desc = &cf.desc;
            let label = &cf.operand.label;
            let default_value = &cf.operand.default_value;
            let hidden = cf.operand.hidden;
            let spread = cf.operand.spread;
            Some(quote! {
                ::ecmd::Positional::from_raw(::ecmd::__private::RawPositional {
                    name: #name,
                    required: #required,
                    desc: #desc,
                    label: #label,
                    default_value: #default_value,
                    hidden: #hidden,
                    spread: #spread,
                })
            })
        })
        .collect();
    let count = defs.len();
    quote! { static POSITIONALS: [::ecmd::Positional; #count] = [#(#defs),*]; }
}

// ── Value assignment (FromStr) ───────────────────────────────────

fn gen_value_assign(id: &Ident, field: &Field, ch: char) -> TokenStream {
    let inner = crate::classify::inner_type_name(&field.ty);
    if inner.as_deref() == Some("String") {
        quote! { #id = Some((*v).to_owned()); }
    } else {
        gen_parse_into(quote! { #id = Some }, ch)
    }
}

fn gen_repeatable_push(id: &Ident, field: &Field, ch: char) -> TokenStream {
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
fn gen_parse_into(target: TokenStream, ch: char) -> TokenStream {
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
        FieldRole::BoolFlag(a) => Some((a.short, quote! { ::ecmd::FlagKind::Bool })),
        FieldRole::PolarityFlag(a) => Some((a.short, quote! { ::ecmd::FlagKind::Polar })),
        FieldRole::ValuedFlag(a) | FieldRole::RepeatableValueFlag(a) => {
            Some((a.short, quote! { ::ecmd::FlagKind::Value }))
        }
        FieldRole::PolarValueFlag(a) => Some((a.short, quote! { ::ecmd::FlagKind::PolarValue })),
        _ => None,
    }
}

fn flag_value_name(role: &FieldRole) -> &str {
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
