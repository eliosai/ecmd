//! The statics one definition is built from

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use super::{ClassifiedField, flag_attrs, flag_def_tokens, flag_value_name};
use crate::attrs::{CommandAttrs, RepeatAttr};
use crate::classify::FieldRole;

/// The statics one definition is built from, ending in a borrow of the definition itself
pub fn gen_meta(
    cmd: &CommandAttrs,
    sections: &crate::attrs::DocSections,
    fields: &[ClassifiedField<'_>],
    policy: &TokenStream,
) -> TokenStream {
    let name = &cmd.name;
    let about = &sections.about;
    let short_doc = &cmd.short_doc;
    let style = style_tokens(cmd);
    let help_style = help_style_tokens(cmd);
    let lenient = cmd.lenient;
    let permute = !cmd.no_permute;
    let flags = gen_flag_metas(cmd, fields);
    let positionals = gen_positional_metas(fields);
    let (rest_static, rest) = gen_rest(fields);
    let tags = gen_tags(cmd);
    let extra_lines: &Vec<String> = if cmd.extra_help.is_empty() {
        &sections.extra
    } else {
        &cmd.extra_help
    };
    let description = texts_static(&quote! { DESCRIPTION }, &sections.description);
    let extra = texts_static(&quote! { EXTRA }, extra_lines);
    let exit_status = texts_static(&quote! { EXIT_STATUS }, &sections.exit_status);
    quote! {
        #flags
        #positionals
        #rest_static
        #policy
        #tags
        #description
        #extra
        #exit_status
        static DEF: ::ecmd::Def = ::ecmd::Def::from_raw(::ecmd::__private::RawDef {
            name: #name,
            about: #about,
            short_doc: #short_doc,
            style: #style,
            help_style: #help_style,
            lenient: #lenient,
            permute: #permute,
            flags: &FLAGS,
            positionals: &POSITIONALS,
            rest: #rest,
            policy: &POLICY,
            tags: &TAGS,
            description: &DESCRIPTION,
            extra: &EXTRA,
            exit_status: &EXIT_STATUS,
        });
        &DEF
    }
}

/// A static array of borrowed text lines under `name`
pub fn texts_static(name: &TokenStream, lines: &[String]) -> TokenStream {
    let count = lines.len();
    quote! {
        static #name: [::std::borrow::Cow<'static, str>; #count] =
            [#(::std::borrow::Cow::Borrowed(#lines)),*];
    }
}

/// A static text array for a flag list, or a bare empty slice when there is nothing to hold
pub fn texts_ref(name: &str, lines: &[String]) -> (TokenStream, TokenStream) {
    if lines.is_empty() {
        return (TokenStream::new(), quote! { &[] });
    }
    let ident = Ident::new(name, Span::call_site());
    (texts_static(&quote! { #ident }, lines), quote! { &#ident })
}

pub fn gen_tags(cmd: &CommandAttrs) -> TokenStream {
    let count = cmd.tags.len();
    let keys = cmd.tags.iter().map(|(key, _)| key);
    let values = cmd.tags.iter().map(|(_, value)| value);
    quote! {
        static TAGS: [::ecmd::__private::Tag; #count] =
            [#(::ecmd::__private::Tag::new_static(#keys, #values)),*];
    }
}

/// The rest slot as a static, and the borrow the definition takes of it
pub fn gen_rest(fields: &[ClassifiedField<'_>]) -> (TokenStream, TokenStream) {
    let Some(cf) = fields.iter().find(|cf| matches!(cf.role, FieldRole::Rest)) else {
        return (TokenStream::new(), quote! { None });
    };
    let name = cf.ident.to_string();
    let desc = &cf.desc;
    let label = &cf.operand.label;
    let default_value = &cf.operand.default_value;
    let hidden = cf.operand.hidden;
    let required = cf.operand.required;
    let rest = quote! {
        static REST: ::ecmd::Positional = ::ecmd::Positional::from_raw(::ecmd::__private::RawPositional {
            name: #name,
            required: #required,
            desc: #desc,
            label: #label,
            default_value: #default_value,
            hidden: #hidden,
            spread: false,
        });
    };
    (rest, quote! { Some(&REST) })
}

/// The `FLAGS` static and the text statics its entries borrow
pub fn gen_flag_metas(cmd: &CommandAttrs, fields: &[ClassifiedField<'_>]) -> TokenStream {
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
    let count = defs.len();
    quote! {
        #(#statics)*
        static FLAGS: [::ecmd::Flag; #count] = [#(#defs),*];
    }
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
        let (text_static, reference) = texts_ref(&format!("FLAG_{index}_{suffix}"), lines);
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

/// GNU long name for a flag: explicit `long=`, else kebab field name in gnu style, else empty.
pub fn flag_long(cf: &ClassifiedField<'_>, cmd: &CommandAttrs) -> String {
    if let Some(attrs) = flag_attrs(&cf.role)
        && let Some(explicit) = &attrs.long
    {
        return explicit.clone();
    }
    if cmd.style == "gnu" {
        return kebab(cf.ident);
    }
    String::new()
}

/// Convert a field ident to a kebab-case long name (`nchars_exact` → "nchars-exact").
pub fn kebab(ident: &Ident) -> String {
    ident.to_string().trim_matches('_').replace('_', "-")
}

/// The parsing style tokens for this command (Gnu when opted in, else Posix).
pub fn style_tokens(cmd: &CommandAttrs) -> TokenStream {
    if cmd.style == "gnu" {
        quote! { ::ecmd::Style::Gnu }
    } else {
        quote! { ::ecmd::Style::Posix }
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

pub fn gen_positional_metas(fields: &[ClassifiedField<'_>]) -> TokenStream {
    let defs: Vec<_> = fields
        .iter()
        .filter_map(|cf| {
            let required = match &cf.role {
                FieldRole::RequiredPositional => true,
                FieldRole::OptionalPositional => false,
                _ => return None,
            };
            let name = cf.ident.to_string();
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
    let count = defs.len();
    quote! { static POSITIONALS: [::ecmd::Positional; #count] = [#(#defs),*]; }
}
