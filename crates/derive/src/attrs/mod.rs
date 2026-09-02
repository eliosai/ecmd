//! Attribute parsing for `#[command(...)]`, `#[flag(...)]` and `#[operand(...)]`

use proc_macro2::Span;
use syn::{Expr, Ident, Lit, Token};

mod doc;
mod flag;
mod operand;

pub use doc::{DocSections, extract_doc_comment, extract_doc_sections};
pub use flag::{FlagAttrs, RepeatAttr};
pub use operand::OperandAttrs;

/// Struct-level command attributes.
#[expect(
    clippy::struct_excessive_bools,
    reason = "derive options are independent compile-time switches"
)]
pub struct CommandAttrs {
    pub name: String,
    pub style: String,
    pub help_style: Option<String>,
    pub lenient: bool,
    pub noop: String,
    pub tags: Vec<(String, String)>,
    pub short_doc: String,
    pub extra_help: Vec<String>,
    pub no_permute: bool,
    pub no_override: bool,
    pub no_implicit_version: bool,
}

impl CommandAttrs {
    pub fn from_ast(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut name = String::new();
        let mut style = "posix".to_owned();
        let mut help_style: Option<String> = None;
        let mut lenient = false;
        let mut noop = String::new();
        let mut tags = Vec::new();
        let mut short_doc = String::new();
        let mut extra_help = Vec::new();
        let mut no_permute = false;
        let mut no_override = false;
        let mut no_implicit_version = false;

        for attr in attrs.iter().filter(|a| a.path().is_ident("command")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    name = parse_lit_str(&meta)?;
                } else if meta.path.is_ident("style") {
                    style = parse_lit_str(&meta)?;
                } else if meta.path.is_ident("help_style") {
                    help_style = Some(parse_lit_str(&meta)?);
                } else if meta.path.is_ident("lenient") {
                    lenient = true;
                } else if meta.path.is_ident("no_permute") {
                    no_permute = true;
                } else if meta.path.is_ident("no_override") {
                    no_override = true;
                } else if meta.path.is_ident("no_implicit_version") {
                    no_implicit_version = true;
                } else if meta.path.is_ident("noop") {
                    noop = parse_lit_str(&meta)?;
                } else if meta.path.is_ident("short_doc") {
                    short_doc = parse_lit_str(&meta)?;
                } else if meta.path.is_ident("extra_help") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    let lits = content.parse_terminated(
                        |input: syn::parse::ParseStream<'_>| input.parse::<syn::LitStr>(),
                        Token![,],
                    )?;
                    extra_help.extend(lits.iter().map(syn::LitStr::value));
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
            let span = attrs
                .first()
                .map_or_else(Span::call_site, |a| a.bracket_token.span.join());
            return Err(syn::Error::new(span, "missing #[command(name = \"...\")]"));
        }

        Ok(Self {
            name,
            style,
            help_style,
            lenient,
            noop,
            tags,
            short_doc,
            extra_help,
            no_permute,
            no_override,
            no_implicit_version,
        })
    }
}

pub fn parse_lit_str(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<String> {
    let value: Expr = meta.value()?.parse()?;
    if let Expr::Lit(syn::ExprLit {
        lit: Lit::Str(s), ..
    }) = &value
    {
        return Ok(s.value());
    }
    Err(meta.error("expected string literal"))
}

pub fn parse_lit_char(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<char> {
    let value: Expr = meta.value()?.parse()?;
    if let Expr::Lit(syn::ExprLit {
        lit: Lit::Char(c), ..
    }) = &value
    {
        return Ok(c.value());
    }
    Err(meta.error("expected char literal"))
}
