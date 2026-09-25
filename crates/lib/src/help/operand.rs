//! Operand fragments of the usage line in each help dialect

use crate::def::Positional;
use crate::style::HelpStyle;

/// One operand as the dialect spells it on the usage line
pub fn usage(style: HelpStyle, operand: &Positional, repeats: bool) -> String {
    let label = operand.shown_as();
    let required = operand.is_required();
    match style {
        HelpStyle::Bash => bash(label, required, repeats),
        HelpStyle::Gnu => gnu(label, required, repeats),
        HelpStyle::Clap | HelpStyle::ClapWide => clap(label, required, repeats),
        HelpStyle::UtilLinux => util_linux(label, required, repeats),
    }
}

fn bash(label: &str, required: bool, repeats: bool) -> String {
    match (required, repeats) {
        (true, false) => label.to_owned(),
        (false, false) => format!("[{label}]"),
        (true, true) => format!("{label} [{label} ...]"),
        (false, true) => format!("[{label} ...]"),
    }
}

fn gnu(label: &str, required: bool, repeats: bool) -> String {
    let tail = if repeats { "..." } else { "" };
    if required {
        format!("{label}{tail}")
    } else {
        format!("[{label}]{tail}")
    }
}

fn clap(label: &str, required: bool, repeats: bool) -> String {
    let tail = if repeats { "..." } else { "" };
    if required {
        format!("<{label}>{tail}")
    } else {
        format!("[{label}]{tail}")
    }
}

fn util_linux(label: &str, required: bool, repeats: bool) -> String {
    let tail = if repeats { "..." } else { "" };
    if required {
        format!("<{label}>{tail}")
    } else {
        format!("[<{label}>{tail}]")
    }
}

#[cfg(test)]
mod tests {
    use super::usage;
    use crate::def::Positional;
    use crate::style::HelpStyle;

    fn spellings(style: HelpStyle) -> [String; 4] {
        let optional = Positional::new("file");
        let required = Positional::new("file").required();
        [
            usage(style, &required, false),
            usage(style, &optional, false),
            usage(style, &required, true),
            usage(style, &optional, true),
        ]
    }

    #[test]
    fn bash_repeats_the_label_inside_its_brackets() {
        assert_eq!(
            spellings(HelpStyle::Bash),
            ["file", "[file]", "file [file ...]", "[file ...]"]
        );
    }

    #[test]
    fn gnu_trails_the_ellipsis_after_the_brackets() {
        assert_eq!(
            spellings(HelpStyle::Gnu),
            ["file", "[file]", "file...", "[file]..."]
        );
    }

    #[test]
    fn clap_angles_a_required_operand() {
        let expected = ["<file>", "[file]", "<file>...", "[file]..."];
        assert_eq!(spellings(HelpStyle::Clap), expected);
        assert_eq!(spellings(HelpStyle::ClapWide), expected);
    }

    #[test]
    fn util_linux_angles_every_operand() {
        assert_eq!(
            spellings(HelpStyle::UtilLinux),
            ["<file>", "[<file>]", "<file>...", "[<file>...]"]
        );
    }
}
