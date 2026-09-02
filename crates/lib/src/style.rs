//! Parsing style controlling which CLI convention layers are active.

/// The argument conventions a command accepts, each style adding to the one before
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Style {
    /// POSIX only: `-x` flags, strict option-before-operand ordering.
    #[default]
    Posix,
    /// POSIX + GNU: `--long` options, `--opt=val`, permutation.
    Gnu,
}

/// The help dialect a command renders, independent of its parsing style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum HelpStyle {
    /// `name: usage` with an indented body, as bash prints its builtins.
    #[default]
    Bash,
    /// `Usage:` line, then tab-separated options, as GNU coreutils prints.
    Gnu,
    /// `Usage:`, `Arguments:`, and a column-aligned `Options:`, as clap prints.
    Clap,
    /// Like [`Self::Clap`], with each description on the line below its label.
    ClapWide,
    /// A leading blank, `Usage:` alone, and a `(1)` trailer, as util-linux prints.
    UtilLinux,
}

impl HelpStyle {
    /// The dialect a command renders when it declares no explicit help style.
    #[must_use]
    pub const fn from_parse_style(style: Style) -> Self {
        match style {
            Style::Posix => Self::Bash,
            Style::Gnu => Self::Gnu,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HelpStyle, Style};

    #[test]
    fn default_is_posix() {
        assert_eq!(Style::default(), Style::Posix);
    }

    #[test]
    fn help_style_defaults_follow_the_parse_style() {
        assert_eq!(HelpStyle::from_parse_style(Style::Posix), HelpStyle::Bash);
        assert_eq!(HelpStyle::from_parse_style(Style::Gnu), HelpStyle::Gnu);
        assert_eq!(HelpStyle::default(), HelpStyle::Bash);
    }

    #[test]
    fn copy_semantics() {
        let s = Style::Gnu;
        let s2 = s;
        assert_eq!(s, s2);
    }
}
