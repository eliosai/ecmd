//! One key and value pair a command carries for its consumers

use std::borrow::Cow;

/// A tag: a key with an optional value
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Tag {
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    key: Cow<'static, str>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    value: Cow<'static, str>,
}

impl Tag {
    pub fn new(key: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    /// A tag over static text, which a derived definition builds in place
    #[must_use]
    pub const fn new_static(key: &'static str, value: &'static str) -> Self {
        Self {
            key: Cow::Borrowed(key),
            value: Cow::Borrowed(value),
        }
    }

    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}
