//! One declared flag

use std::borrow::Cow;

use super::raw::RawFlag;
use super::text;

/// How a flag consumes arguments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum FlagKind {
    /// Presence sets it
    #[default]
    Bool,
    /// Takes one value, attached or from the next argument
    Value,
    /// Tracks `-x` against `+x`
    Polar,
    /// Tracks `-x` against `+x` and takes a value
    PolarValue,
    /// Accepted and dropped
    Noop,
}

/// A flag the scanner recognizes, from its identity to how help shows it
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each bool is one independent trait of the flag"
)]
pub struct Flag {
    id: char,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    long: Cow<'static, str>,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Texts))]
    aliases: Cow<'static, [Cow<'static, str>]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Code))]
    kind: FlagKind,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    desc: Cow<'static, str>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    value_name: Cow<'static, str>,
    hidden: bool,
    implemented: bool,
    repeatable: bool,
    allow_hyphen_values: bool,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Texts))]
    possible_values: Cow<'static, [Cow<'static, str>]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Texts))]
    help_values: Cow<'static, [Cow<'static, str>]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    default_value: Cow<'static, str>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    help_label: Cow<'static, str>,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Texts))]
    visible_aliases: Cow<'static, [Cow<'static, str>]>,
}

/// The first identity a long-only flag takes, past every byte-derived short
pub const SYNTHETIC_BASE: char = '\u{E000}';

impl Flag {
    /// A boolean flag spelled `-short`
    #[must_use]
    pub const fn new(short: char) -> Self {
        Self {
            id: short,
            long: Cow::Borrowed(""),
            aliases: Cow::Borrowed(&[]),
            kind: FlagKind::Bool,
            desc: Cow::Borrowed(""),
            value_name: Cow::Borrowed(""),
            hidden: false,
            implemented: true,
            repeatable: true,
            allow_hyphen_values: true,
            possible_values: Cow::Borrowed(&[]),
            help_values: Cow::Borrowed(&[]),
            default_value: Cow::Borrowed(""),
            help_label: Cow::Borrowed(""),
            visible_aliases: Cow::Borrowed(&[]),
        }
    }

    /// A boolean flag spelled `--long` alone, given its identity when a definition takes it
    #[must_use]
    pub fn long_only(long: impl Into<Cow<'static, str>>) -> Self {
        Self::new('\0').long(long)
    }

    /// Name the flag `--long` as well
    #[must_use]
    pub fn long(mut self, long: impl Into<Cow<'static, str>>) -> Self {
        self.long = long.into();
        self
    }

    /// Answer to another long name that help does not list
    #[must_use]
    pub fn alias(mut self, alias: impl Into<Cow<'static, str>>) -> Self {
        self.aliases.to_mut().push(alias.into());
        self
    }

    /// Answer to another long name that help lists
    #[must_use]
    pub fn visible_alias(mut self, alias: impl Into<Cow<'static, str>>) -> Self {
        self.visible_aliases.to_mut().push(alias.into());
        self
    }

    /// Set how the flag consumes arguments
    #[must_use]
    pub const fn kind(mut self, kind: FlagKind) -> Self {
        self.kind = kind;
        self
    }

    /// Take one value, shown in help under this name
    #[must_use]
    pub fn value(mut self, value_name: impl Into<Cow<'static, str>>) -> Self {
        self.value_name = value_name.into();
        self.kind = match self.kind {
            FlagKind::Polar | FlagKind::PolarValue => FlagKind::PolarValue,
            FlagKind::Bool | FlagKind::Value | FlagKind::Noop => FlagKind::Value,
        };
        self
    }

    /// Describe the flag in help
    #[must_use]
    pub fn desc(mut self, desc: impl Into<Cow<'static, str>>) -> Self {
        self.desc = desc.into();
        self
    }

    /// Keep the flag out of help
    #[must_use]
    pub const fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Declare the flag but leave it to an external implementation
    #[must_use]
    pub const fn unimplemented(mut self) -> Self {
        self.implemented = false;
        self
    }

    /// Refuse a second occurrence under GNU conventions
    #[must_use]
    pub const fn once(mut self) -> Self {
        self.repeatable = false;
        self
    }

    /// Refuse a separated value that opens with a hyphen
    #[must_use]
    pub const fn reject_hyphen_values(mut self) -> Self {
        self.allow_hyphen_values = false;
        self
    }

    /// Accept only these values
    #[must_use]
    pub fn possible_values(
        mut self,
        values: impl IntoIterator<Item = impl Into<Cow<'static, str>>>,
    ) -> Self {
        self.possible_values = values.into_iter().map(Into::into).collect();
        self
    }

    /// List these values in help
    #[must_use]
    pub fn help_values(
        mut self,
        values: impl IntoIterator<Item = impl Into<Cow<'static, str>>>,
    ) -> Self {
        self.help_values = values.into_iter().map(Into::into).collect();
        self
    }

    /// Note this default in help
    #[must_use]
    pub fn default_value(mut self, value: impl Into<Cow<'static, str>>) -> Self {
        self.default_value = value.into();
        self
    }

    /// Replace the generated help label
    #[must_use]
    pub fn help_label(mut self, label: impl Into<Cow<'static, str>>) -> Self {
        self.help_label = label.into();
        self
    }

    /// The identity a scan reports the flag under
    #[must_use]
    pub const fn id(&self) -> char {
        self.id
    }

    /// The short character, or none for a long-only flag
    #[must_use]
    pub const fn short(&self) -> Option<char> {
        if self.id < SYNTHETIC_BASE && self.id != '\0' {
            Some(self.id)
        } else {
            None
        }
    }

    /// The long name, or none for a short-only flag
    #[must_use]
    pub fn long_name(&self) -> Option<&str> {
        text(&self.long)
    }

    /// The long names help does not list
    pub fn aliases(&self) -> impl ExactSizeIterator<Item = &str> {
        self.aliases.iter().map(AsRef::as_ref)
    }

    /// The long names help lists beside the primary one
    pub fn visible_aliases(&self) -> impl ExactSizeIterator<Item = &str> {
        self.visible_aliases.iter().map(AsRef::as_ref)
    }

    /// Every long name the flag answers to
    pub fn long_names(&self) -> impl Iterator<Item = &str> {
        self.long_name().into_iter().chain(self.aliases()).chain(
            self.visible_aliases()
                .filter(|alias| alias.chars().count() > 1),
        )
    }

    /// Whether the flag answers to this long name
    #[must_use]
    pub fn answers_to(&self, name: &str) -> bool {
        self.long_names().any(|long| long == name)
    }

    /// How the flag consumes arguments
    #[must_use]
    pub const fn flag_kind(&self) -> FlagKind {
        self.kind
    }

    /// Whether the flag takes a value
    #[must_use]
    pub const fn takes_value(&self) -> bool {
        matches!(self.kind, FlagKind::Value | FlagKind::PolarValue)
    }

    /// The help description
    #[must_use]
    pub fn description(&self) -> &str {
        &self.desc
    }

    /// The name help shows for the value
    #[must_use]
    pub fn value_name(&self) -> Option<&str> {
        text(&self.value_name)
    }

    /// Whether help omits the flag
    #[must_use]
    pub const fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Whether the command implements the flag itself
    #[must_use]
    pub const fn is_implemented(&self) -> bool {
        self.implemented
    }

    /// Whether a second occurrence is accepted under GNU conventions
    #[must_use]
    pub const fn is_repeatable(&self) -> bool {
        self.repeatable
    }

    /// Whether a separated value may open with a hyphen
    #[must_use]
    pub const fn allows_hyphen_values(&self) -> bool {
        self.allow_hyphen_values
    }

    /// The values the flag accepts, empty when any value does
    pub fn accepted_values(&self) -> impl ExactSizeIterator<Item = &str> {
        self.possible_values.iter().map(AsRef::as_ref)
    }

    /// The values help lists
    pub fn listed_values(&self) -> impl ExactSizeIterator<Item = &str> {
        self.help_values.iter().map(AsRef::as_ref)
    }

    /// The default help notes
    #[must_use]
    pub fn default(&self) -> Option<&str> {
        text(&self.default_value)
    }

    /// The authored help label
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        text(&self.help_label)
    }

    /// The identity a long-only flag takes at this position in its definition
    pub fn assign_id(&mut self, position: usize) {
        if self.id == '\0' {
            let offset = u32::try_from(position).unwrap_or(0);
            self.id = char::from_u32(u32::from(SYNTHETIC_BASE).saturating_add(offset))
                .unwrap_or(SYNTHETIC_BASE);
        }
    }
}

impl Flag {
    #[doc(hidden)]
    #[must_use]
    pub const fn from_raw(raw: RawFlag) -> Self {
        Self {
            id: raw.id,
            long: Cow::Borrowed(raw.long),
            aliases: Cow::Borrowed(raw.aliases),
            kind: raw.kind,
            desc: Cow::Borrowed(raw.desc),
            value_name: Cow::Borrowed(raw.value_name),
            hidden: raw.hidden,
            implemented: raw.implemented,
            repeatable: raw.repeatable,
            allow_hyphen_values: raw.allow_hyphen_values,
            possible_values: Cow::Borrowed(raw.possible_values),
            help_values: Cow::Borrowed(raw.help_values),
            default_value: Cow::Borrowed(raw.default_value),
            help_label: Cow::Borrowed(raw.help_label),
            visible_aliases: Cow::Borrowed(raw.visible_aliases),
        }
    }
}
