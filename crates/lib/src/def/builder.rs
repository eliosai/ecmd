//! Building a definition at runtime

use std::borrow::Cow;

use super::{Def, Flag, Policy, Positional, Tag};
use crate::style::{HelpStyle, Style};

/// A definition under construction
#[derive(Debug, Clone)]
pub struct DefBuilder {
    def: Def,
}

impl DefBuilder {
    /// A definition for the command called `name`, with no flags yet
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            def: Def {
                name: name.into(),
                about: Cow::Borrowed(""),
                short_doc: Cow::Borrowed(""),
                style: Style::Posix,
                help_style: HelpStyle::Bash,
                lenient: false,
                permute: true,
                flags: Cow::Borrowed(&[]),
                positionals: Cow::Borrowed(&[]),
                rest: None,
                policy: Cow::Borrowed(&Policy::STANDARD),
                tags: Cow::Borrowed(&[]),
                description: Cow::Borrowed(&[]),
                extra: Cow::Borrowed(&[]),
                exit_status: Cow::Borrowed(&[]),
            },
        }
    }

    /// The one-line summary help opens with
    #[must_use]
    pub fn about(mut self, about: impl Into<Cow<'static, str>>) -> Self {
        self.def.about = about.into();
        self
    }

    /// An authored usage line in place of the generated one
    #[must_use]
    pub fn short_doc(mut self, short_doc: impl Into<Cow<'static, str>>) -> Self {
        self.def.short_doc = short_doc.into();
        self
    }

    /// The argument conventions the command accepts, which also picks the help dialect
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.def.style = style;
        self.def.help_style = HelpStyle::from_parse_style(style);
        self
    }

    /// The help dialect, when it differs from the one the style implies
    #[must_use]
    pub const fn help_style(mut self, help_style: HelpStyle) -> Self {
        self.def.help_style = help_style;
        self
    }

    /// Turn an unknown flag into an operand instead of an error
    #[must_use]
    pub const fn lenient(mut self) -> Self {
        self.def.lenient = true;
        self
    }

    /// Whether flags may follow operands
    #[must_use]
    pub const fn permute(mut self, permute: bool) -> Self {
        self.def.permute = permute;
        self
    }

    /// Require exact long option names instead of unambiguous prefixes
    #[must_use]
    pub fn exact_long(mut self) -> Self {
        self.def.policy.to_mut().exact_long = true;
        self
    }

    /// Withhold the implicit `-V` short for `--version`
    #[must_use]
    pub fn no_implicit_version(mut self) -> Self {
        self.def.policy.to_mut().no_implicit_version = true;
        self
    }

    /// Add one flag, giving a long-only flag its identity
    #[must_use]
    pub fn flag(mut self, mut flag: Flag) -> Self {
        let flags = self.def.flags.to_mut();
        flag.assign_id(flags.len());
        flags.push(flag);
        self
    }

    /// Add every flag in order
    #[must_use]
    pub fn flags(self, flags: impl IntoIterator<Item = Flag>) -> Self {
        flags.into_iter().fold(self, Self::flag)
    }

    /// Add one positional slot after the ones declared so far
    #[must_use]
    pub fn positional(mut self, positional: Positional) -> Self {
        self.def.positionals.to_mut().push(positional);
        self
    }

    /// Take every operand past the positionals into this slot
    #[must_use]
    pub fn rest(mut self, rest: Positional) -> Self {
        self.def.rest = Some(Cow::Owned(rest));
        self
    }

    /// Carry one key and value pair for consumers
    #[must_use]
    pub fn tag(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.def.tags.to_mut().push(Tag::new(key, value));
        self
    }

    /// Carry every key and value pair in order
    #[must_use]
    pub fn tags(
        self,
        tags: impl IntoIterator<Item = (impl Into<Cow<'static, str>>, impl Into<Cow<'static, str>>)>,
    ) -> Self {
        tags.into_iter()
            .fold(self, |builder, (key, value)| builder.tag(key, value))
    }

    /// Add every positional slot in order
    #[must_use]
    pub fn positionals(self, positionals: impl IntoIterator<Item = Positional>) -> Self {
        positionals.into_iter().fold(self, Self::positional)
    }

    /// The body lines between the summary and the options
    #[must_use]
    pub fn description(
        mut self,
        lines: impl IntoIterator<Item = impl Into<Cow<'static, str>>>,
    ) -> Self {
        self.def.description = lines.into_iter().map(Into::into).collect();
        self
    }

    /// The lines placed after the options
    #[must_use]
    pub fn extra(mut self, lines: impl IntoIterator<Item = impl Into<Cow<'static, str>>>) -> Self {
        self.def.extra = lines.into_iter().map(Into::into).collect();
        self
    }

    /// The lines of the exit status section
    #[must_use]
    pub fn exit_status(
        mut self,
        lines: impl IntoIterator<Item = impl Into<Cow<'static, str>>>,
    ) -> Self {
        self.def.exit_status = lines.into_iter().map(Into::into).collect();
        self
    }

    /// The finished definition
    #[must_use]
    pub fn build(self) -> Def {
        self.def
    }
}
