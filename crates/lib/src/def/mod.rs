//! What a command accepts and how it presents itself

use std::borrow::Cow;

use crate::error::Error;
use crate::parse::Scan;
use crate::style::{HelpStyle, Style};

mod builder;
mod flag;
mod policy;
mod positional;
mod raw;
mod tag;

pub use builder::DefBuilder;
pub use flag::{Flag, FlagKind};
pub use policy::{ExclusiveRule, NumericRule, Policy, ValueMode, ValueRule};
pub use positional::Positional;
pub use raw::{RawDef, RawFlag, RawPositional};
pub use tag::Tag;

/// A command definition: its flags, operands, parsing policy and help text
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Def {
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    name: Cow<'static, str>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    about: Cow<'static, str>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    short_doc: Cow<'static, str>,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Code))]
    style: Style,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Code))]
    help_style: HelpStyle,
    lenient: bool,
    permute: bool,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    flags: Cow<'static, [Flag]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    positionals: Cow<'static, [Positional]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::Map<crate::archive::Owned>))]
    rest: Option<Cow<'static, Positional>>,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Owned))]
    policy: Cow<'static, Policy>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    tags: Cow<'static, [Tag]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Texts))]
    description: Cow<'static, [Cow<'static, str>]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Texts))]
    extra: Cow<'static, [Cow<'static, str>]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Texts))]
    exit_status: Cow<'static, [Cow<'static, str>]>,
}

impl Def {
    /// Start a definition for the command called `name`
    pub fn builder(name: impl Into<Cow<'static, str>>) -> DefBuilder {
        DefBuilder::new(name)
    }

    /// The command name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The one-line summary, empty when the command has none
    #[must_use]
    pub fn about(&self) -> &str {
        &self.about
    }

    /// The authored usage line that replaces the generated one
    #[must_use]
    pub fn short_doc(&self) -> Option<&str> {
        text(&self.short_doc)
    }

    /// The argument conventions the command accepts
    #[must_use]
    pub const fn style(&self) -> Style {
        self.style
    }

    /// The help dialect the command renders
    #[must_use]
    pub const fn help_style(&self) -> HelpStyle {
        self.help_style
    }

    /// Whether an unknown flag becomes an operand instead of an error
    #[must_use]
    pub const fn lenient(&self) -> bool {
        self.lenient
    }

    /// Whether flags may follow operands
    #[must_use]
    pub const fn permute(&self) -> bool {
        self.permute
    }

    /// Every declared flag in declaration order
    #[must_use]
    pub fn flags(&self) -> &[Flag] {
        &self.flags
    }

    /// The flag with this identity
    #[must_use]
    pub fn flag(&self, id: char) -> Option<&Flag> {
        self.flags.iter().find(|flag| flag.id() == id)
    }

    /// The flag a one-character short or a long name, alias or visible alias refers to
    #[must_use]
    pub fn flag_named(&self, name: &str) -> Option<&Flag> {
        let mut chars = name.chars();
        match (chars.next(), chars.next()) {
            (Some(short), None) => self.flags.iter().find(|flag| flag.short() == Some(short)),
            _ => self.flags.iter().find(|flag| flag.answers_to(name)),
        }
    }

    /// Every declared positional in order
    #[must_use]
    pub fn positionals(&self) -> &[Positional] {
        &self.positionals
    }

    /// The slot that takes every operand past the positionals
    #[must_use]
    pub fn rest(&self) -> Option<&Positional> {
        self.rest.as_deref()
    }

    /// Every tag as a key and value pair, in declaration order
    #[must_use]
    pub fn tags(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.tags.iter().map(|tag| (tag.key(), tag.value()))
    }

    /// The value of the first tag with this key
    #[must_use]
    pub fn tag(&self, key: &str) -> Option<&str> {
        self.tags
            .iter()
            .find_map(|tag| (tag.key() == key).then(|| tag.value()))
    }

    /// The body lines between the summary and the options
    pub fn description(&self) -> impl ExactSizeIterator<Item = &str> {
        self.description.iter().map(AsRef::as_ref)
    }

    /// The lines placed after the options
    pub fn extra(&self) -> impl ExactSizeIterator<Item = &str> {
        self.extra.iter().map(AsRef::as_ref)
    }

    /// The lines of the exit status section
    pub fn exit_status(&self) -> impl ExactSizeIterator<Item = &str> {
        self.exit_status.iter().map(AsRef::as_ref)
    }

    /// The same definition under another name
    #[must_use]
    pub fn with_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// The same definition with one more tag
    #[must_use]
    pub fn with_tag(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.tags.to_mut().push(Tag::new(key, value));
        self
    }

    /// Scan one argument slice against this definition
    pub fn scan<'a>(&'a self, args: &'a [&'a str]) -> Result<Scan<'a>, Error> {
        crate::parse::scan(self, args)
    }

    #[doc(hidden)]
    #[must_use]
    pub fn policy(&self) -> &Policy {
        &self.policy
    }
}

/// A borrowed string, or none when it is empty
pub fn text(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}
