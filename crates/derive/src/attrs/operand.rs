//! `#[operand(...)]` on a positional or rest field

use syn::Field;

use super::parse_lit_str;

/// Help presentation for a positional or rest field, from `#[operand(...)]`.
#[derive(Default)]
pub struct OperandAttrs {
    pub label: String,
    pub default_value: String,
    pub hidden: bool,
    pub required: bool,
    pub spread: bool,
}

impl OperandAttrs {
    pub fn from_field(field: &Field) -> syn::Result<Self> {
        let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("operand")) else {
            return Ok(Self::default());
        };
        let mut attrs = Self::default();
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("label") {
                attrs.label = parse_lit_str(&meta)?;
            } else if meta.path.is_ident("default") {
                attrs.default_value = parse_lit_str(&meta)?;
            } else if meta.path.is_ident("hide") {
                attrs.hidden = true;
            } else if meta.path.is_ident("required") {
                attrs.required = true;
            } else if meta.path.is_ident("spread") {
                attrs.spread = true;
            } else {
                return Err(meta.error("unknown operand attribute"));
            }
            Ok(())
        })?;
        Ok(attrs)
    }
}
