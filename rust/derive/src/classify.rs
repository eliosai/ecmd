//! Field classification — determines parse behavior from types.

use syn::{Field, GenericArgument, Ident, PathArguments, Type};

use crate::attrs::FlagAttrs;

/// What role a field plays in parsing.
pub enum FieldRole {
    /// Boolean flag: `-x` sets true.
    BoolFlag(FlagAttrs),
    /// Polarity flag: `-x` On, `+x` Off.
    PolarityFlag(FlagAttrs),
    /// Valued flag: `-x val` produces Option<T>.
    ValuedFlag(FlagAttrs),
    /// Repeatable valued flag: `-x a -x b` produces Vec<String>.
    RepeatableValueFlag(FlagAttrs),
    /// Accumulated polar values: `-o val` / `+o val`.
    PolarValueFlag(FlagAttrs),
    /// Optional positional (Option<String>).
    OptionalPositional,
    /// Required positional (String).
    RequiredPositional,
    /// Rest-of-args collector (Operands type).
    Rest,
}

/// Classify a struct field into its parsing role.
pub fn classify_field(field: &Field) -> syn::Result<FieldRole> {
    let flag_attrs = FlagAttrs::from_field(field)?;
    let ty_name = outer_type_name(&field.ty);

    match flag_attrs {
        Some(attrs) => Ok(classify_flagged(attrs, &ty_name, &field.ty)),
        None if ty_name == "Operands" => Ok(FieldRole::Rest),
        None if ty_name == "Option" => Ok(FieldRole::OptionalPositional),
        None => Ok(FieldRole::RequiredPositional),
    }
}

fn classify_flagged(attrs: FlagAttrs, ty_name: &str, ty: &Type) -> FieldRole {
    match ty_name {
        "bool" => FieldRole::BoolFlag(attrs),
        "Polarity" => FieldRole::PolarityFlag(attrs),
        "Vec" => classify_vec(attrs, ty),
        _ => FieldRole::ValuedFlag(attrs),
    }
}

fn classify_vec(attrs: FlagAttrs, ty: &Type) -> FieldRole {
    let inner = inner_type_name(ty);
    if inner.as_deref() == Some("PolarVal") {
        FieldRole::PolarValueFlag(attrs)
    } else {
        FieldRole::RepeatableValueFlag(attrs)
    }
}

/// Extract the outermost type name (e.g. "Option", "Vec", "bool").
fn outer_type_name(ty: &Type) -> String {
    if let Type::Path(path) = ty
        && let Some(seg) = path.path.segments.last()
    {
        return seg.ident.to_string();
    }
    String::new()
}

/// Extract the inner type name from `Vec<T>` or `Option<T>`.
pub fn inner_type_name(ty: &Type) -> Option<String> {
    let Type::Path(path) = ty else { return None };
    let seg = path.path.segments.last()?;
    let PathArguments::AngleBracketed(args) = &seg.arguments else { return None };
    let GenericArgument::Type(inner) = args.args.first()? else { return None };
    Some(outer_type_name(inner))
}

/// Get the field ident (named fields only).
///
/// # Panics
///
/// Panics if called on a tuple struct field (upstream validates this).
#[expect(clippy::panic, reason = "proc macro invariant: only named structs")]
pub fn field_ident(field: &Field) -> &Ident {
    field.ident.as_ref().unwrap_or_else(|| {
        panic!("ecmd derive: only named struct fields are supported")
    })
}
