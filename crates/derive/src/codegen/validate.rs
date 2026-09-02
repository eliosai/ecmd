//! The shape checks every derive input passes before code is written

use std::collections::HashSet;

use syn::Ident;

use super::{ClassifiedField, clears_targets, flag_attrs};
use crate::classify::FieldRole;

pub fn validate(fields: &[ClassifiedField<'_>]) -> syn::Result<()> {
    check_duplicate_flags(fields)?;
    check_operands_last(fields)?;
    check_positional_ordering(fields)?;
    check_clears_targets(fields)
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
