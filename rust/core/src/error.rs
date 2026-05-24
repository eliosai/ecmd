//! Parse errors produced when command-line arguments don't match
//! the declared command structure.

use std::fmt;

/// An error encountered while parsing command-line arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// An unrecognized flag character or long option was provided.
    UnknownFlag(String),
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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFlag(flag) => write!(f, "{flag}: invalid option"),
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
    fn missing_required_displays_correctly() {
        let e = Error::MissingRequired("target".into());
        assert_eq!(e.to_string(), "missing required argument: target");
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
