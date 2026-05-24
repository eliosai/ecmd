//! Flag polarity for shell-style `+x` / `-x` semantics.
//!
//! In POSIX shells, some builtins accept `+flag` to turn OFF and `-flag`
//! to turn ON. `Polarity` captures this three-state: unset, on, or off.

/// Three-state flag polarity.
///
/// - `Unset` — flag was not provided (default)
/// - `On` — flag was provided with `-` prefix
/// - `Off` — flag was provided with `+` prefix
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[expect(clippy::exhaustive_enums, reason = "closed domain: on/off/unset")]
pub enum Polarity {
    /// Flag was not provided.
    #[default]
    Unset,
    /// Flag was enabled (`-x`).
    On,
    /// Flag was disabled (`+x`).
    Off,
}

impl Polarity {
    /// Whether the flag was explicitly turned on.
    #[must_use]
    pub const fn is_on(self) -> bool {
        matches!(self, Self::On)
    }

    /// Whether the flag was explicitly turned off.
    #[must_use]
    pub const fn is_off(self) -> bool {
        matches!(self, Self::Off)
    }

    /// Whether the flag was provided at all (on or off).
    #[must_use]
    pub const fn is_set(self) -> bool {
        !matches!(self, Self::Unset)
    }
}

/// A valued flag that records its polarity.
///
/// Used for flags like `set -o errexit` / `set +o errexit` where
/// each occurrence carries both a polarity and a string value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[expect(clippy::exhaustive_structs, reason = "simple value object with pub fields")]
pub struct PolarVal {
    /// Whether this was `-flag` (On) or `+flag` (Off).
    pub polarity: Polarity,
    /// The value associated with this flag occurrence.
    pub value: String,
}

impl PolarVal {
    /// Create a new polar value.
    #[must_use]
    pub fn new(polarity: Polarity, value: impl Into<String>) -> Self {
        Self {
            polarity,
            value: value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PolarVal, Polarity};

    #[test]
    fn default_is_unset() {
        assert_eq!(Polarity::default(), Polarity::Unset);
    }

    #[test]
    fn on_reports_correctly() {
        let p = Polarity::On;
        assert!(p.is_on());
        assert!(!p.is_off());
        assert!(p.is_set());
    }

    #[test]
    fn off_reports_correctly() {
        let p = Polarity::Off;
        assert!(!p.is_on());
        assert!(p.is_off());
        assert!(p.is_set());
    }

    #[test]
    fn unset_is_not_set() {
        let p = Polarity::Unset;
        assert!(!p.is_on());
        assert!(!p.is_off());
        assert!(!p.is_set());
    }

    #[test]
    fn polar_val_stores_on_with_value() {
        let pv = PolarVal::new(Polarity::On, "errexit");
        assert_eq!(pv.polarity, Polarity::On);
        assert_eq!(pv.value, "errexit");
    }

    #[test]
    fn polar_val_stores_off_with_value() {
        let pv = PolarVal::new(Polarity::Off, "verbose");
        assert!(pv.polarity.is_off());
        assert_eq!(pv.value, "verbose");
    }

    #[test]
    fn polarity_is_copy() {
        let p = Polarity::On;
        let p2 = p;
        assert_eq!(p, p2);
    }

    #[test]
    fn polar_val_equality() {
        let a = PolarVal::new(Polarity::On, "x");
        let b = PolarVal::new(Polarity::On, "x");
        let c = PolarVal::new(Polarity::Off, "x");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
