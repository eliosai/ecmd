//! The `POLICY` static from the `tag(...)` keys

use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::quote;

use super::meta::kebab;
use super::{ClassifiedField, flag_attrs};
use crate::attrs::CommandAttrs;
use crate::classify::FieldRole;

pub struct ValuePolicyLiteral {
    ch: char,
    mode: TokenStream,
    default: String,
}

pub fn gen_policy(cmd: &CommandAttrs, fields: &[ClassifiedField<'_>]) -> syn::Result<TokenStream> {
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

pub fn append_flag_identities(
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

pub fn append_exclusive_groups(
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

pub fn append_value_rules(
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

pub fn append_numeric_operands(
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

pub fn policy_field<'a, 'field>(
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

pub fn policy_any_field<'a, 'field>(
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

pub fn unique_policy_long<'a, 'field>(
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

pub fn policy_long(field: &ClassifiedField<'_>, name: &str) -> bool {
    flag_attrs(&field.role).is_some_and(|attrs| {
        attrs.long.as_deref() == Some(name) || (attrs.long.is_none() && kebab(field.ident) == name)
    })
}

pub struct PolicySpec<'a> {
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
pub fn policy_literal(spec: &PolicySpec<'_>) -> TokenStream {
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

pub fn identities_literal(identities: &[char]) -> TokenStream {
    quote! { &[#(#identities),*] }
}
