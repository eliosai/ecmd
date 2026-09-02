//! Copying static metadata into its runtime-owned form

use crate::parse::FlagDef;
use crate::policy::ValueRule;

use super::{CommandDef, Owned, OwnedCommandDef, PositionalDef, Static};

impl CommandDef<Static> {
    /// Copy static metadata into its runtime-owned form
    pub fn into_owned(self) -> OwnedCommandDef {
        OwnedCommandDef {
            name: self.name.to_owned(),
            about: self.about.to_owned(),
            short_doc: self.short_doc.to_owned(),
            style: self.style,
            help_style: self.help_style,
            on_unknown: self.on_unknown,
            permute: self.permute,
            flags: self
                .flags
                .iter()
                .cloned()
                .map(FlagDef::into_owned)
                .collect(),
            positionals: self
                .positionals
                .iter()
                .cloned()
                .map(PositionalDef::into_owned)
                .collect(),
            has_rest: self.has_rest,
            rest_label: self.rest_label.to_owned(),
            rest_hidden: self.rest_hidden,
            rest_desc: self.rest_desc.to_owned(),
            rest_default: self.rest_default.to_owned(),
            rest_required: self.rest_required,
            value_rules: self
                .value_rules
                .iter()
                .cloned()
                .map(ValueRule::into_owned)
                .collect(),
            numeric_operands: self.numeric_operands.to_vec(),
            first_numeric_value: self.first_numeric_value,
            exact_long: self.exact_long,
            no_implicit_version: self.no_implicit_version,
            equals_only: self.equals_only.to_vec(),
            attached_values: self.attached_values.to_vec(),
            separated_values: self.separated_values.to_vec(),
            prefixed_values: self.prefixed_values.to_vec(),
            exclusive_groups: self.exclusive_groups.to_vec(),
            tags: self
                .tags
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect(),
            description: self
                .description
                .iter()
                .map(|line| (*line).to_owned())
                .collect(),
            extra: self.extra.iter().map(|line| (*line).to_owned()).collect(),
            exit_status: self
                .exit_status
                .iter()
                .map(|line| (*line).to_owned())
                .collect(),
        }
    }
}

impl PositionalDef<Static> {
    fn into_owned(self) -> PositionalDef<Owned> {
        PositionalDef {
            name: self.name.to_owned(),
            required: self.required,
            desc: self.desc.to_owned(),
            label: self.label.to_owned(),
            default_value: self.default_value.to_owned(),
            hidden: self.hidden,
            spread: self.spread,
        }
    }
}
