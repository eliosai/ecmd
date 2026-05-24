//! Parsing style controlling which CLI convention layers are active.

/// Controls which argument parsing conventions are applied.
///
/// Each style is additive — GNU includes POSIX, Modern includes GNU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Style {
    /// POSIX only: `-x` flags, strict option-before-operand ordering.
    #[default]
    Posix,
    /// POSIX + GNU: `--long` options, `--opt=val`, permutation.
    Gnu,
}

#[cfg(test)]
mod tests {
    use super::Style;

    #[test]
    fn default_is_posix() {
        assert_eq!(Style::default(), Style::Posix);
    }

    #[test]
    fn copy_semantics() {
        let s = Style::Gnu;
        let s2 = s;
        assert_eq!(s, s2);
    }
}
