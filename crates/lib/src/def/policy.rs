//! The value, numeric and exclusivity rules one command adds to the standard scan

use std::borrow::Cow;

/// How a valued flag behaves when no value is attached
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValueMode {
    /// Use the default and leave the next argument untouched
    AttachedOrDefault,
    /// Consume a following operand or use the default
    NextOrDefault,
    /// Consume a following numeric value or use the default
    NumericNextOrDefault,
    /// Optionally consume a following numeric value or use the default
    OptionalNumericNextOrDefault,
    /// Use the default only for an exact short option
    ExactShortDefault,
    /// Consume any following token or use the default
    AnyNextOrDefault,
}

/// Missing-value behavior for one declared flag
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[expect(clippy::exhaustive_structs, reason = "derive plumbing")]
pub struct ValueRule {
    pub ch: char,
    #[cfg_attr(feature = "rkyv", rkyv(with = crate::archive::Code))]
    pub mode: ValueMode,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    pub default: Cow<'static, str>,
}

/// Legacy numeric operand routed into a valued flag
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    rkyv(attr(non_exhaustive))
)]
#[expect(clippy::exhaustive_structs, reason = "derive plumbing")]
pub struct NumericRule {
    pub ch: char,
    pub prefix: char,
}

/// One flag's membership in an exclusive option group
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    rkyv(attr(non_exhaustive))
)]
#[expect(clippy::exhaustive_structs, reason = "derive plumbing")]
pub struct ExclusiveRule {
    pub ch: char,
    pub group: u16,
}

/// Every rule a command adds to the standard scan
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[expect(clippy::exhaustive_structs, reason = "derive plumbing")]
pub struct Policy {
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    pub value_rules: Cow<'static, [ValueRule]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    pub numeric_operands: Cow<'static, [NumericRule]>,
    pub first_numeric_value: Option<char>,
    pub exact_long: bool,
    pub no_implicit_version: bool,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    pub equals_only: Cow<'static, [char]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    pub attached_values: Cow<'static, [char]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    pub separated_values: Cow<'static, [char]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    pub prefixed_values: Cow<'static, [char]>,
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    pub exclusive_groups: Cow<'static, [ExclusiveRule]>,
}

impl Policy {
    /// The policy that adds nothing
    pub const STANDARD: Self = Self {
        value_rules: Cow::Borrowed(&[]),
        numeric_operands: Cow::Borrowed(&[]),
        first_numeric_value: None,
        exact_long: false,
        no_implicit_version: false,
        equals_only: Cow::Borrowed(&[]),
        attached_values: Cow::Borrowed(&[]),
        separated_values: Cow::Borrowed(&[]),
        prefixed_values: Cow::Borrowed(&[]),
        exclusive_groups: Cow::Borrowed(&[]),
    };

    #[must_use]
    pub fn value(&self, ch: char) -> Option<&ValueRule> {
        self.value_rules.iter().find(|rule| rule.ch == ch)
    }

    #[must_use]
    pub fn requires_equals(&self, ch: char) -> bool {
        self.equals_only.contains(&ch)
    }

    #[must_use]
    pub fn requires_attached(&self, ch: char) -> bool {
        self.attached_values.contains(&ch)
    }

    #[must_use]
    pub fn requires_separated(&self, ch: char) -> bool {
        self.separated_values.contains(&ch)
    }

    #[must_use]
    pub fn accepts_prefix(&self, ch: char) -> bool {
        self.prefixed_values.contains(&ch)
    }

    #[must_use]
    pub fn exclusive_group(&self, ch: char) -> Option<u16> {
        self.exclusive_groups
            .iter()
            .find_map(|rule| (rule.ch == ch).then_some(rule.group))
    }
}
