//! The shape checks every derive input passes before code is written

use std::collections::HashSet;

use syn::Ident;

use super::{ClassifiedField, clears_targets, flag_attrs};
use crate::classify::FieldRole;

pub fn validate(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
    check_duplicate_flags(fields)?;
    check_operands_last(fields)?;
    check_positional_ordering(fields)?;
    check_operand_attrs(fields)?;
    check_clears_targets(fields)
}

/// `required` belongs to the rest slot alone and a required positional carries no default
pub fn check_operand_attrs(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
    for cf in fields {
        if let Some(message) = misplaced_operand_attr(cf) {
            return Err(syn::Error::new_spanned(cf.field, message));
        }
    }
    Ok(())
}

fn misplaced_operand_attr(cf: &ClassifiedField<'_>) -> Option<&'static str> {
    let positional = matches!(
        cf.role,
        FieldRole::OptionalPositional | FieldRole::RequiredPositional
    );
    let tagged = cf.field.attrs.iter().any(|a| a.path().is_ident("operand"));
    match &cf.role {
        FieldRole::Rest => None,
        _ if positional && cf.operand.required => {
            Some("a positional is required by its type; `required` applies to Operands")
        }
        FieldRole::RequiredPositional if !cf.operand.default_value.is_empty() => {
            Some("a required positional has no default; make the field an Option")
        }
        _ if !positional && tagged => {
            Some("`operand` applies to a positional or Operands field, not a flag")
        }
        _ => None,
    }
}

pub fn check_positional_ordering(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
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

pub fn check_duplicate_flags(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
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

pub fn check_operands_last(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
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

pub fn check_clears_targets(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
    let flag_names: HashSet<&Ident> = fields
        .iter()
        .filter(|cf| flag_attrs(&cf.role).is_some())
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
