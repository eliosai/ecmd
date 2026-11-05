//! The cells one definition is built from

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use super::cell::{array_cell, cell, texts_cell};
use super::{ClassifiedField, flag_attrs, flag_def_tokens, flag_value_name};
use crate::attrs::{CommandAttrs, RepeatAttr};
use crate::classify::FieldRole;

/// The once-built definition and every cell it borrows, ending in a borrow of the definition
pub fn gen_meta(
    cmd: &CommandAttrs,
    sections: &crate::attrs::DocSections,
    fields: &[ClassifiedField<'_>],
    policy: &(TokenStream, TokenStream),
) -> TokenStream {
    let (flag_items, flags) = gen_flag_metas(cmd, fields);
    let (positional_items, positionals) = gen_positional_metas(fields);
    let (rest_item, rest) = gen_rest(fields);
    let (policy_items, policy) = policy;
    let (tag_items, tags) = gen_tags(cmd);
    let (text_items, texts) = gen_texts(cmd, sections);
    let raw = raw_def(cmd, sections);
    let body = quote! {
        #flag_items #positional_items #rest_item #policy_items #tag_items #text_items
        ::ecmd::Def::from_raw(::ecmd::__private::RawDef {
            flags: #flags, positionals: #positionals, rest: #rest, policy: #policy, tags: #tags,
            #texts
            #raw
        })
    };
    let (item, borrow) = cell(&ident("DEF"), &quote! { ::ecmd::Def }, &body);
    quote! { #item #borrow }
}

/// The scalar fields of the raw definition
fn raw_def(cmd: &CommandAttrs, sections: &crate::attrs::DocSections) -> TokenStream {
    let name = &cmd.name;
    let about = &sections.about;
    let short_doc = &cmd.short_doc;
    let style = style_tokens(cmd);
    let help_style = help_style_tokens(cmd);
    let lenient = cmd.lenient;
    let permute = !cmd.no_permute;
    quote! {
        name: #name,
        about: #about,
        short_doc: #short_doc,
        style: #style,
        help_style: #help_style,
        lenient: #lenient,
        permute: #permute,
    }
}

/// The description, extra and exit status cells, and the raw fields that borrow them
fn gen_texts(
    cmd: &CommandAttrs,
    sections: &crate::attrs::DocSections,
) -> (TokenStream, TokenStream) {
    let extra_lines: &Vec<String> = if cmd.extra_help.is_empty() {
        &sections.extra
    } else {
        &cmd.extra_help
    };
    let (description_item, description) = texts_cell(&ident("DESCRIPTION"), &sections.description);
    let (extra_item, extra) = texts_cell(&ident("EXTRA"), extra_lines);
    let (exit_item, exit_status) = texts_cell(&ident("EXIT_STATUS"), &sections.exit_status);
    let items = quote! { #description_item #extra_item #exit_item };
    let fields = quote! {
        description: #description,
        extra: #extra,
        exit_status: #exit_status,
    };
    (items, fields)
}

/// An identifier spelled `name` at the call site
pub fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

/// The tags cell and the expression that borrows it
pub fn gen_tags(cmd: &CommandAttrs) -> (TokenStream, TokenStream) {
    let tags: Vec<_> = cmd
        .tags
        .iter()
        .map(|(key, value)| quote! { ::ecmd::__private::Tag::new_static(#key, #value) })
        .collect();
    array_cell(&ident("TAGS"), &quote! { ::ecmd::__private::Tag }, &tags)
}

/// The rest slot as a cell, and the borrow the definition takes of it
pub fn gen_rest(fields: &[ClassifiedField<'_>]) -> (TokenStream, TokenStream) {
    let Some(cf) = fields.iter().find(|cf| matches!(cf.role, FieldRole::Rest)) else {
        return (TokenStream::new(), quote! { None });
    };
    let name = operand_name(cf.ident);
    let desc = &cf.desc;
    let label = &cf.operand.label;
    let default_value = &cf.operand.default_value;
    let hidden = cf.operand.hidden;
    let required = cf.operand.required;
    let value = quote! {
        ::ecmd::Positional::from_raw(::ecmd::__private::RawPositional {
            name: #name,
            required: #required,
            desc: #desc,
            label: #label,
            default_value: #default_value,
            hidden: #hidden,
            spread: false,
        })
    };
    let (item, borrow) = cell(&ident("REST"), &quote! { ::ecmd::Positional }, &value);
    (item, quote! { Some(#borrow) })
}

/// The flags cell with the text cells its entries borrow, and the expression that borrows it
pub fn gen_flag_metas(
    cmd: &CommandAttrs,
    fields: &[ClassifiedField<'_>],
) -> (TokenStream, TokenStream) {
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
    let (item, borrow) = array_cell(&ident("FLAGS"), &quote! { ::ecmd::Flag }, &defs);
    (quote! { #(#statics)* #item }, borrow)
}

/// One `Flag::from_raw` literal for a flag field, its text lists pushed as statics
pub fn flag_def_literal(
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
        let (text_static, reference) = texts_cell(&ident(&format!("FLAG_{index}_{suffix}")), lines);
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

/// GNU long name for a flag: explicit `long=`, else kebab field name in gnu or clap style, else empty.
pub fn flag_long(cf: &ClassifiedField<'_>, cmd: &CommandAttrs) -> String {
    if let Some(attrs) = flag_attrs(&cf.role)
        && let Some(explicit) = &attrs.long
    {
        return explicit.clone();
    }
    if matches!(cmd.style.as_str(), "gnu" | "clap") {
        return kebab(cf.ident);
    }
    String::new()
}

/// The operand name a field ident shows in help and errors, without a leading underscore
pub fn operand_name(ident: &Ident) -> String {
    ident.to_string().trim_start_matches('_').to_owned()
}

/// Convert a field ident to a kebab-case long name (`nchars_exact` → "nchars-exact").
pub fn kebab(ident: &Ident) -> String {
    ident.to_string().trim_matches('_').replace('_', "-")
}

/// The parsing style tokens for this command (Gnu or Clap when opted in, else Posix).
pub fn style_tokens(cmd: &CommandAttrs) -> TokenStream {
    match cmd.style.as_str() {
        "gnu" => quote! { ::ecmd::Style::Gnu },
        "clap" => quote! { ::ecmd::Style::Clap },
        _ => quote! { ::ecmd::Style::Posix },
    }
}

/// The declared help dialect, defaulting to the one implied by the parse style.
pub fn help_style_tokens(cmd: &CommandAttrs) -> TokenStream {
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

/// The positionals cell and the expression that borrows it
pub fn gen_positional_metas(fields: &[ClassifiedField<'_>]) -> (TokenStream, TokenStream) {
    let defs: Vec<_> = fields
        .iter()
        .filter_map(|cf| {
            let required = match &cf.role {
                FieldRole::RequiredPositional => true,
                FieldRole::OptionalPositional => false,
                _ => return None,
            };
            let name = operand_name(cf.ident);
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
    array_cell(&ident("POSITIONALS"), &quote! { ::ecmd::Positional }, &defs)
}
