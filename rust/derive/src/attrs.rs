//! Attribute parsing for `#[command(...)]` and `#[flag(...)]`.

use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::{Expr, Field, Ident, Lit, Token};

/// Struct-level command attributes.
pub struct CommandAttrs {
    pub name: String,
    pub style: String,
    pub lenient: bool,
    pub noop: String,
    pub tags: Vec<(String, String)>,
}

impl CommandAttrs {
    pub fn from_ast(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut name = String::new();
        let mut style = "posix".to_owned();
        let mut lenient = false;
        let mut noop = String::new();
        let mut tags = Vec::new();

        for attr in attrs.iter().filter(|a| a.path().is_ident("command")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    name = parse_lit_str(&meta)?;
                } else if meta.path.is_ident("style") {
                    style = parse_lit_str(&meta)?;
                } else if meta.path.is_ident("lenient") {
                    lenient = true;
                } else if meta.path.is_ident("noop") {
                    noop = parse_lit_str(&meta)?;
                } else if meta.path.is_ident("tag") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    let key: Ident = content.parse()?;
                    let value = if content.peek(Token![=]) {
                        content.parse::<Token![=]>()?;
                        let lit: syn::LitStr = content.parse()?;
                        lit.value()
                    } else {
                        String::new()
                    };
                    tags.push((key.to_string(), value));
                } else {
                    return Err(meta.error("unknown command attribute"));
                }
                Ok(())
            })?;
        }

        if name.is_empty() {
            let span = attrs.first()
                .map_or_else(Span::call_site, |a| a.bracket_token.span.join());
            return Err(syn::Error::new(span, "missing #[command(name = \"...\")]"));
        }

        Ok(Self { name, style, lenient, noop, tags })
    }
}

/// Field-level flag attributes.
pub struct FlagAttrs {
    pub short: char,
    pub clears: Vec<Ident>,
    pub value_name: String,
}

impl FlagAttrs {
    pub fn from_field(field: &Field) -> syn::Result<Option<Self>> {
        let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("flag")) else {
            return Ok(None);
        };

        let mut short = '\0';
        let mut clears = Vec::new();
        let mut value_name = String::new();

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("short") {
                short = parse_lit_char(&meta)?;
            } else if meta.path.is_ident("clears") {
                let content;
                syn::parenthesized!(content in meta.input);
                let idents = content.parse_terminated(Ident::parse_any, Token![,])?;
                clears.extend(idents);
            } else if meta.path.is_ident("value_name") {
                value_name = parse_lit_str(&meta)?;
            } else {
                return Err(meta.error("unknown flag attribute"));
            }
            Ok(())
        })?;

        if short == '\0' {
            return Err(syn::Error::new_spanned(attr, "missing `short` in #[flag(...)]"));
        }

        Ok(Some(Self { short, clears, value_name }))
    }
}

/// Extract doc comment lines from attributes, trimmed and joined.
pub fn extract_doc_comment(attrs: &[syn::Attribute]) -> String {
    let lines: Vec<String> = attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| {
            if let syn::Meta::NameValue(nv) = &a.meta {
                if let Expr::Lit(syn::ExprLit { lit: Lit::Str(s), .. }) = &nv.value {
                    return Some(s.value());
                }
            }
            None
        })
        .map(|s| s.strip_prefix(' ').unwrap_or(&s).to_owned())
        .collect();
    lines.join("\n").trim().to_owned()
}

fn parse_lit_str(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<String> {
    let value: Expr = meta.value()?.parse()?;
    if let Expr::Lit(syn::ExprLit { lit: Lit::Str(s), .. }) = &value {
        return Ok(s.value());
    }
    Err(meta.error("expected string literal"))
}

fn parse_lit_char(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<char> {
    let value: Expr = meta.value()?.parse()?;
    if let Expr::Lit(syn::ExprLit { lit: Lit::Char(c), .. }) = &value {
        return Ok(c.value());
    }
    Err(meta.error("expected char literal"))
}
