//! One positional argument slot

use std::borrow::Cow;

use super::raw::RawPositional;
use super::text;

/// A positional argument slot and how help shows it
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Positional {
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    name: Cow<'static, str>,
    required: bool,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    desc: Cow<'static, str>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    label: Cow<'static, str>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    default_value: Cow<'static, str>,
    hidden: bool,
    spread: bool,
}

impl Positional {
    /// An optional slot called `name`
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            required: false,
            desc: Cow::Borrowed(""),
            label: Cow::Borrowed(""),
            default_value: Cow::Borrowed(""),
            hidden: false,
            spread: false,
        }
    }

    /// Fail the parse when the slot is empty
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Describe the slot in help
    #[must_use]
    pub fn desc(mut self, desc: impl Into<Cow<'static, str>>) -> Self {
        self.desc = desc.into();
        self
    }

    /// Show the slot under this label instead of its name
    #[must_use]
    pub fn label(mut self, label: impl Into<Cow<'static, str>>) -> Self {
        self.label = label.into();
        self
    }

    /// Note this default in help
    #[must_use]
    pub fn default_value(mut self, value: impl Into<Cow<'static, str>>) -> Self {
        self.default_value = value.into();
        self
    }

    /// Keep the slot out of help
    #[must_use]
    pub const fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Show the slot as taking more than one value
    #[must_use]
    pub const fn spread(mut self) -> Self {
        self.spread = true;
        self
    }

    /// The slot name, which errors report
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the parse fails when the slot is empty
    #[must_use]
    pub const fn is_required(&self) -> bool {
        self.required
    }

    /// The help description
    #[must_use]
    pub fn description(&self) -> &str {
        &self.desc
    }

    /// The label help shows, which is the name unless one was authored
    #[must_use]
    pub fn shown_as(&self) -> &str {
        text(&self.label).unwrap_or(&self.name)
    }

    /// The default help notes
    #[must_use]
    pub fn default(&self) -> Option<&str> {
        text(&self.default_value)
    }

    /// Whether help omits the slot
    #[must_use]
    pub const fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Whether help shows the slot as taking more than one value
    #[must_use]
    pub const fn is_spread(&self) -> bool {
        self.spread
    }
}

impl Positional {
    #[doc(hidden)]
    #[must_use]
    pub const fn from_raw(raw: RawPositional) -> Self {
        Self {
            name: Cow::Borrowed(raw.name),
            required: raw.required,
            desc: Cow::Borrowed(raw.desc),
            label: Cow::Borrowed(raw.label),
            default_value: Cow::Borrowed(raw.default_value),
            hidden: raw.hidden,
            spread: raw.spread,
        }
    }
}
