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
    pub short_doc: String,
    pub extra_help: Vec<String>,
    pub no_permute: bool,
}

impl CommandAttrs {
    pub fn from_ast(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut name = String::new();
        let mut style = "posix".to_owned();
        let mut lenient = false;
        let mut noop = String::new();
        let mut tags = Vec::new();
        let mut short_doc = String::new();
        let mut extra_help = Vec::new();
        let mut no_permute = false;

        for attr in attrs.iter().filter(|a| a.path().is_ident("command")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    name = parse_lit_str(&meta)?;
                } else if meta.path.is_ident("style") {
                    style = parse_lit_str(&meta)?;
                } else if meta.path.is_ident("lenient") {
                    lenient = true;
                } else if meta.path.is_ident("no_permute") {
                    no_permute = true;
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
            let span = attrs.first()
                .map_or_else(Span::call_site, |a| a.bracket_token.span.join());
            return Err(syn::Error::new(span, "missing #[command(name = \"...\")]"));
        }

        Ok(Self { name, style, lenient, noop, tags, short_doc, extra_help, no_permute })
    }
}

/// Field-level flag attributes.
pub struct FlagAttrs {
    pub short: char,
    pub clears: Vec<Ident>,
    pub value_name: String,
    pub long: Option<String>,
    pub aliases: Vec<String>,
    pub hidden: bool,
}

impl FlagAttrs {
    pub fn from_field(field: &Field) -> syn::Result<Option<Self>> {
        let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("flag")) else {
            return Ok(None);
        };

        let mut short = '\0';
        let mut clears = Vec::new();
        let mut value_name = String::new();
        let mut long = None;
        let mut aliases = Vec::new();
        let mut hidden = false;

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
            } else if meta.path.is_ident("long") {
                long = Some(parse_lit_str(&meta)?);
            } else if meta.path.is_ident("alias") {
                aliases.push(parse_lit_str(&meta)?);
            } else if meta.path.is_ident("hide") {
                hidden = true;
            } else {
                return Err(meta.error("unknown flag attribute"));
            }
            Ok(())
        })?;

        if short == '\0' && long.is_none() {
            return Err(syn::Error::new_spanned(
                attr,
                "#[flag(...)] needs `short` or `long`",
            ));
        }

        Ok(Some(Self { short, clears, value_name, long, aliases, hidden }))
    }
}

/// Parsed doc comment split into help sections.
pub struct DocSections {
    pub about: String,
    pub description: Vec<String>,
    pub extra: Vec<String>,
    pub exit_status: Vec<String>,
}

/// Extract and split doc comment into bash-compatible help sections.
pub fn extract_doc_sections(attrs: &[syn::Attribute]) -> DocSections {
    let lines = extract_doc_lines(attrs);
    parse_sections(&lines)
}

/// Extract doc comment lines from attributes, trimmed and joined.
pub fn extract_doc_comment(attrs: &[syn::Attribute]) -> String {
    let lines = extract_doc_lines(attrs);
    lines.join("\n").trim().to_owned()
}

fn extract_doc_lines(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| {
            if let syn::Meta::NameValue(nv) = &a.meta
                && let Expr::Lit(syn::ExprLit { lit: Lit::Str(s), .. }) = &nv.value
            {
                return Some(s.value());
            }
            None
        })
        .map(|s| s.strip_prefix(' ').unwrap_or(&s).to_owned())
        .collect()
}

#[derive(PartialEq)]
enum DocState { About, Description, Extra, ExitStatus }

fn parse_sections(lines: &[String]) -> DocSections {
    let mut about = String::new();
    let mut description = Vec::new();
    let mut extra = Vec::new();
    let mut exit_status = Vec::new();

    let mut state = DocState::About;
    let mut past_about_blank = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed == "# Options" {
            state = DocState::Extra;
            continue;
        }
        if trimmed == "# Exit Status" {
            state = DocState::ExitStatus;
            continue;
        }

        match state {
            DocState::About => {
                if trimmed.is_empty() {
                    if !about.is_empty() {
                        past_about_blank = true;
                    }
                } else if past_about_blank {
                    state = DocState::Description;
                    description.push(trimmed.to_owned());
                } else if about.is_empty() {
                    trimmed.clone_into(&mut about);
                } else {
                    about.push(' ');
                    about.push_str(trimmed);
                }
            },
            DocState::Description => {
                description.push(if trimmed.is_empty() { String::new() } else { trimmed.to_owned() });
            },
            DocState::Extra => {
                extra.push(if trimmed.is_empty() { String::new() } else { trimmed.to_owned() });
            },
            DocState::ExitStatus => {
                exit_status.push(if trimmed.is_empty() { String::new() } else { trimmed.to_owned() });
            },
        }
    }

    trim_trailing_empty(&mut description);
    trim_trailing_empty(&mut extra);
    trim_trailing_empty(&mut exit_status);

    DocSections { about, description, extra, exit_status }
}

fn trim_trailing_empty(lines: &mut Vec<String>) {
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
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
