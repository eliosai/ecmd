//! Parse errors produced when command-line arguments don't match
//! the declared command structure.

use std::fmt;

/// An error encountered while parsing command-line arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// An unrecognized flag character or long option was provided.
    UnknownFlag(String),
    /// A declared flag is available only through an external command.
    UnimplementedFlag(String),
    /// A flag that requires a value was not given one.
    MissingValue(String),
    /// A required positional argument was not provided.
    MissingRequired(String),
    /// A flag's value could not be parsed into the expected type.
    InvalidValue {
        /// The flag that received the bad value.
        flag: String,
        /// The value that was provided.
        value: String,
        /// Why the value was rejected.
        reason: String,
    },
    /// An unrecognized subcommand name was provided.
    UnknownCommand(String),
    /// An abbreviated long option matched more than one declared option.
    AmbiguousOption(String),
    /// A flag that takes no value was given one via `--flag=value`.
    UnexpectedValue {
        /// The flag that rejected the value.
        flag: String,
        /// The value that was provided.
        value: String,
    },
    /// A scalar flag was supplied more than once.
    RepeatedFlag(String),
    /// A flag was combined with a mutually exclusive flag.
    ConflictingFlags {
        /// The flag that created the conflict.
        current: String,
        /// The previously parsed flag.
        previous: String,
    },
    /// An obsolete numeric value appeared after the first argument.
    FirstNumericValue {
        /// The option character reported by the command.
        option: char,
        /// The regular valued flag that replaces the obsolete form.
        flag: char,
        /// The obsolete value placeholder.
        value_name: String,
    },
    /// `--help` was requested; the caller should print help and exit 0.
    HelpRequested,
    /// `--version` was requested; the caller should print the version and exit 0.
    VersionRequested,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFlag(flag) => write!(f, "{flag}: invalid option"),
            Self::UnimplementedFlag(flag) => write!(f, "{flag}: external implementation required"),
            Self::MissingValue(flag) => {
                write!(f, "{flag}: option requires an argument")
            }
            Self::MissingRequired(name) => {
                write!(f, "missing required argument: {name}")
            }
            Self::InvalidValue {
                flag,
                value,
                reason,
            } => write!(f, "{flag}: {value}: {reason}"),
            Self::UnknownCommand(name) => write!(f, "{name}: unknown command"),
            Self::AmbiguousOption(name) => write!(f, "{name}: option is ambiguous"),
            Self::UnexpectedValue { flag, .. } => {
                write!(f, "{flag}: option doesn't allow an argument")
            }
            Self::RepeatedFlag(name) => write!(f, "{name}: option cannot be used multiple times"),
            Self::ConflictingFlags { current, previous } => {
                write!(f, "{current}: conflicts with {previous}")
            }
            Self::FirstNumericValue {
                option,
                flag,
                value_name,
            } => write!(
                f,
                "invalid option -- {option}; -{value_name} is recognized only when it is the first\noption; use -{flag} N instead"
            ),
            Self::HelpRequested => write!(f, "help requested"),
            Self::VersionRequested => write!(f, "version requested"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::Error;

    #[test]
    fn unknown_flag_displays_correctly() {
        let e = Error::UnknownFlag("-z".into());
        assert_eq!(e.to_string(), "-z: invalid option");
    }

    #[test]
    fn missing_value_displays_correctly() {
        let e = Error::MissingValue("-o".into());
        assert_eq!(e.to_string(), "-o: option requires an argument");
    }

    #[test]
    fn unimplemented_flag_displays_correctly() {
        let error = Error::UnimplementedFlag("--ftp-port".into());
        assert_eq!(
            error.to_string(),
            "--ftp-port: external implementation required"
        );
    }

    #[test]
    fn missing_required_displays_correctly() {
        let e = Error::MissingRequired("target".into());
        assert_eq!(e.to_string(), "missing required argument: target");
    }

    #[test]
    fn repeated_flag_displays_correctly() {
        let error = Error::RepeatedFlag("--verbose".into());
        assert_eq!(
            error.to_string(),
            "--verbose: option cannot be used multiple times"
        );
    }

    #[test]
    fn invalid_value_displays_correctly() {
        let e = Error::InvalidValue {
            flag: "-n".into(),
            value: "abc".into(),
            reason: "not a number".into(),
        };
        assert_eq!(e.to_string(), "-n: abc: not a number");
    }

    #[test]
    fn unknown_command_displays_correctly() {
        let e = Error::UnknownCommand("frobnicate".into());
        assert_eq!(e.to_string(), "frobnicate: unknown command");
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<Error>();
    }

    #[test]
    fn equality_works() {
        let a = Error::UnknownFlag("-x".into());
        let b = Error::UnknownFlag("-x".into());
        let c = Error::UnknownFlag("-y".into());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
