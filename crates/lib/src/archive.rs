//! rkyv wrappers for the enums and nested text lists a definition holds

use core::fmt;
use std::borrow::Cow;

use rkyv::rancor::{Fallible, Source};
use rkyv::ser::{Allocator, Writer};
use rkyv::string::ArchivedString;
use rkyv::vec::{ArchivedVec, VecResolver};
use rkyv::with::{ArchiveWith, DeserializeWith, SerializeWith};
use rkyv::{Archive, Deserialize, Place, Serialize};

use crate::def::{FlagKind, ValueMode};
use crate::style::{HelpStyle, Style};

/// A fieldless enum stored as one byte
trait Coded: Sized {
    fn code(&self) -> u8;
    fn from_code(code: u8) -> Option<Self>;
}

/// Archives a fieldless enum as its byte code
pub struct Code;

impl<T: Coded> ArchiveWith<T> for Code {
    type Archived = u8;
    type Resolver = ();

    fn resolve_with(field: &T, (): (), out: Place<u8>) {
        field.code().resolve((), out);
    }
}

impl<T: Coded, S: Fallible + ?Sized> SerializeWith<T, S> for Code {
    fn serialize_with(_: &T, _: &mut S) -> Result<(), S::Error> {
        Ok(())
    }
}

impl<T: Coded, D: Fallible + ?Sized> DeserializeWith<u8, T, D> for Code
where
    D::Error: Source,
{
    fn deserialize_with(field: &u8, _: &mut D) -> Result<T, D::Error> {
        T::from_code(*field).ok_or_else(|| D::Error::new(UnknownCode(*field)))
    }
}

/// A byte that names no variant of the enum it was read into
#[derive(Debug)]
struct UnknownCode(u8);

impl fmt::Display for UnknownCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown enum code {}", self.0)
    }
}

impl core::error::Error for UnknownCode {}

macro_rules! coded {
    ($ty:ty { $($variant:ident = $code:literal),+ $(,)? }) => {
        impl Coded for $ty {
            fn code(&self) -> u8 {
                match self {
                    $(Self::$variant => $code,)+
                }
            }

            fn from_code(code: u8) -> Option<Self> {
                match code {
                    $($code => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

coded!(Style { Posix = 0, Gnu = 1 });
coded!(HelpStyle { Bash = 0, Gnu = 1, Clap = 2, ClapWide = 3, UtilLinux = 4 });
coded!(FlagKind { Bool = 0, Value = 1, Polar = 2, PolarValue = 3, Noop = 4 });
coded!(ValueMode {
    AttachedOrDefault = 0,
    NextOrDefault = 1,
    NumericNextOrDefault = 2,
    OptionalNumericNextOrDefault = 3,
    ExactShortDefault = 4,
    AnyNextOrDefault = 5,
});

/// Archives a `Cow` of one archivable value and reads it back as an owned one
pub struct Owned;

impl<T: Archive + Clone> ArchiveWith<Cow<'static, T>> for Owned {
    type Archived = T::Archived;
    type Resolver = T::Resolver;

    fn resolve_with(field: &Cow<'static, T>, resolver: T::Resolver, out: Place<T::Archived>) {
        T::resolve(field, resolver, out);
    }
}

impl<T: Serialize<S> + Clone, S: Fallible + ?Sized> SerializeWith<Cow<'static, T>, S> for Owned {
    fn serialize_with(
        field: &Cow<'static, T>,
        serializer: &mut S,
    ) -> Result<T::Resolver, S::Error> {
        T::serialize(field, serializer)
    }
}

impl<T, D> DeserializeWith<T::Archived, Cow<'static, T>, D> for Owned
where
    T: Archive + Clone,
    T::Archived: Deserialize<T, D>,
    D: Fallible + ?Sized,
{
    fn deserialize_with(
        field: &T::Archived,
        deserializer: &mut D,
    ) -> Result<Cow<'static, T>, D::Error> {
        field.deserialize(deserializer).map(Cow::Owned)
    }
}

/// Text lines a definition holds
type Lines = Cow<'static, [Cow<'static, str>]>;

/// Archives a list of text lines as a vector of strings
pub struct Texts;

impl ArchiveWith<Lines> for Texts {
    type Archived = ArchivedVec<ArchivedString>;
    type Resolver = VecResolver;

    fn resolve_with(field: &Lines, resolver: VecResolver, out: Place<Self::Archived>) {
        ArchivedVec::resolve_from_len(field.len(), resolver, out);
    }
}

impl<S: Fallible + Allocator + Writer + ?Sized> SerializeWith<Lines, S> for Texts
where
    S::Error: Source,
{
    fn serialize_with(field: &Lines, serializer: &mut S) -> Result<VecResolver, S::Error> {
        ArchivedVec::serialize_from_iter::<String, _, _>(
            field.iter().map(ToString::to_string),
            serializer,
        )
    }
}

impl<D: Fallible + ?Sized> DeserializeWith<ArchivedVec<ArchivedString>, Lines, D> for Texts {
    fn deserialize_with(field: &ArchivedVec<ArchivedString>, _: &mut D) -> Result<Lines, D::Error> {
        Ok(Cow::Owned(
            field
                .iter()
                .map(|line| Cow::Owned(line.as_str().to_owned()))
                .collect(),
        ))
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "tests")]
mod tests {
    use rkyv::rancor::Error;

    use crate::def::{ArchivedDef, Def, Flag, Positional};
    use crate::style::{HelpStyle, Style};

    fn sample() -> Def {
        Def::builder("sample")
            .about("a sample")
            .style(Style::Gnu)
            .help_style(HelpStyle::Clap)
            .lenient()
            .permute(false)
            .exact_long()
            .no_implicit_version()
            .flag(Flag::new('f').long("file").value("PATH").unimplemented())
            .flag(Flag::long_only("verbose").alias("loud").once())
            .positional(Positional::new("source").required())
            .rest(Positional::new("files").spread())
            .tag("kind", "extension")
            .description(["first", "", "third"])
            .extra(["tail"])
            .exit_status(["zero"])
            .build()
    }

    #[test]
    fn a_definition_survives_an_archive_round_trip() {
        let def = sample();
        let bytes = rkyv::to_bytes::<Error>(&def).expect("serialize");
        let archived = rkyv::access::<ArchivedDef, Error>(&bytes).expect("access");
        let decoded = rkyv::deserialize::<Def, Error>(archived).expect("deserialize");
        assert_eq!(decoded, def);
        assert_eq!(decoded.help(), def.help());
    }

    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, PartialEq)]
    struct Holder {
        #[rkyv(with = super::Code)]
        style: Style,
    }

    #[test]
    fn an_unknown_enum_code_is_refused() {
        let holder = Holder { style: Style::Gnu };
        let mut bytes = rkyv::to_bytes::<Error>(&holder).expect("serialize");
        let archived = rkyv::access::<ArchivedHolder, Error>(&bytes).expect("access");
        let offset = std::ptr::from_ref(&archived.style).addr() - bytes.as_ptr().addr();
        if let Some(byte) = bytes.get_mut(offset) {
            *byte = 200;
        }
        let archived = rkyv::access::<ArchivedHolder, Error>(&bytes).expect("access");
        let decoded = rkyv::deserialize::<Holder, Error>(archived).ok();
        assert_eq!(decoded, None);
    }
}
