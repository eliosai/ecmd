//! `#[flag(...)]` on a flag field

use syn::ext::IdentExt;
use syn::{Field, Ident, Token};

use super::{parse_lit_char, parse_lit_str};

/// Field-level flag attributes.
pub struct FlagAttrs {
    pub short: char,
    pub clears: Vec<Ident>,
    pub value_name: String,
    pub long: Option<String>,
    pub aliases: Vec<String>,
    pub hidden: bool,
    pub implemented: bool,
    pub repeat: RepeatAttr,
    pub allow_hyphen_values: bool,
    pub possible_values: Vec<String>,
    pub help_values: Vec<String>,
    pub default_value: String,
    pub help_label: String,
    pub visible_aliases: Vec<String>,
}

/// Per-flag override for command occurrence policy
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RepeatAttr {
    Default,
    Repeatable,
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
        let mut implemented = true;
        let mut repeat = RepeatAttr::Default;
        let mut allow_hyphen_values = true;
        let mut possible_values: Vec<String> = Vec::new();
        let mut help_values: Vec<String> = Vec::new();
        let mut default_value = String::new();
        let mut help_label = String::new();
        let mut visible_aliases: Vec<String> = Vec::new();

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("help_values") {
                let content;
                syn::parenthesized!(content in meta.input);
                let items = content
                    .parse_terminated(<syn::LitStr as syn::parse::Parse>::parse, Token![,])?;
                help_values.extend(items.into_iter().map(|item| item.value()));
            } else if meta.path.is_ident("values") {
                let content;
                syn::parenthesized!(content in meta.input);
                let items = content
                    .parse_terminated(<syn::LitStr as syn::parse::Parse>::parse, Token![,])?;
                possible_values.extend(items.into_iter().map(|item| item.value()));
            } else if meta.path.is_ident("default") {
                default_value = parse_lit_str(&meta)?;
            } else if meta.path.is_ident("help_label") {
                help_label = parse_lit_str(&meta)?;
            } else if meta.path.is_ident("visible_alias") {
                visible_aliases.push(parse_lit_str(&meta)?);
            } else if meta.path.is_ident("short") {
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
            } else if meta.path.is_ident("unimplemented") {
                implemented = false;
            } else if meta.path.is_ident("repeatable") {
                repeat = RepeatAttr::Repeatable;
            } else if meta.path.is_ident("reject_hyphen_values") {
                allow_hyphen_values = false;
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

        Ok(Some(Self {
            short,
            clears,
            value_name,
            long,
            aliases,
            hidden,
            implemented,
            repeat,
            allow_hyphen_values,
            possible_values,
            help_values,
            default_value,
            help_label,
            visible_aliases,
        }))
    }
}
