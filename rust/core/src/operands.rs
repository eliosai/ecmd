//! Remaining positional arguments after flags and named positionals.
//!
//! Use `Operands` as a field type — the derive macro recognizes it
//! and generates "consume remaining args" code automatically.

use std::ops::Deref;

/// All positional arguments remaining after flag parsing completes.
///
/// Derefs to `[String]` for slice access. Use as a struct field
/// with no attribute — the type itself signals "rest of args."
///
/// `Hash` is not derived because `Vec<String>` does not implement it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Operands {
    items: Vec<String>,
}

impl Operands {
    /// Build from any iterable of string-like values.
    #[must_use]
    pub fn from_args<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            items: args.into_iter().map(|s| s.as_ref().to_owned()).collect(),
        }
    }

    /// Number of operands.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether no operands were provided.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Join all operands with a separator.
    #[must_use]
    pub fn join(&self, sep: &str) -> String {
        self.items.join(sep)
    }

    /// First operand, if any.
    #[must_use]
    pub fn first(&self) -> Option<&str> {
        self.items.first().map(String::as_str)
    }

    /// Operand at position `n`, if it exists.
    #[must_use]
    pub fn get(&self, n: usize) -> Option<&str> {
        self.items.get(n).map(String::as_str)
    }
}

impl From<Vec<String>> for Operands {
    fn from(items: Vec<String>) -> Self {
        Self { items }
    }
}

impl Deref for Operands {
    type Target = [String];

    fn deref(&self) -> &[String] {
        &self.items
    }
}

impl<'a> IntoIterator for &'a Operands {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl IntoIterator for Operands {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing, reason = "tests use controlled inputs")]
mod tests {
    use pretty_assertions::assert_eq;

    use super::Operands;

    #[test]
    fn empty_operands_has_zero_length() {
        let ops = Operands::default();
        assert!(ops.is_empty());
        assert_eq!(ops.len(), 0);
    }

    #[test]
    fn default_equals_empty_args() {
        let default = Operands::default();
        let empty: Operands = Operands::from_args(Vec::<&str>::new());
        assert_eq!(default, empty);
    }

    #[test]
    fn collects_from_str_slices() {
        let ops = Operands::from_args(["hello", "world"]);
        assert_eq!(ops.len(), 2);
        assert_eq!(ops.first(), Some("hello"));
        assert_eq!(ops.get(0), Some("hello"));
        assert_eq!(ops.get(1), Some("world"));
        assert_eq!(ops.get(2), None);
    }

    #[test]
    fn from_vec_string_avoids_clone() {
        let v = vec!["a".to_owned(), "b".to_owned()];
        let ops = Operands::from(v);
        assert_eq!(ops.get(0), Some("a"));
        assert_eq!(ops.get(1), Some("b"));
    }

    #[test]
    fn derefs_to_string_slice() {
        let ops = Operands::from_args(["a", "b", "c"]);
        let slice: &[String] = &ops;
        assert_eq!(slice, &["a", "b", "c"]);
    }

    #[test]
    fn join_concatenates_with_separator() {
        let ops = Operands::from_args(["echo", "hello", "world"]);
        assert_eq!(ops.join(" "), "echo hello world");
    }

    #[test]
    fn join_empty_produces_empty_string() {
        let ops = Operands::default();
        assert_eq!(ops.join(" "), "");
    }

    #[test]
    fn iterates_borrowed() {
        let ops = Operands::from_args(["x", "y"]);
        let collected: Vec<&String> = (&ops).into_iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], "x");
    }

    #[test]
    fn iterates_owned() {
        let ops = Operands::from_args(["a", "b"]);
        let collected: Vec<String> = ops.into_iter().collect();
        assert_eq!(collected, vec!["a", "b"]);
    }

    #[test]
    fn clone_produces_equal_value() {
        let ops = Operands::from_args(["a", "b"]);
        let cloned = ops.clone();
        assert_eq!(ops, cloned);
    }
}
