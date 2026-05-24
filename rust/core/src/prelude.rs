//! Single import for ecmd users: `use ecmd::prelude::*;`

#[doc(inline)]
pub use crate::meta::Command as CommandTrait;
#[doc(inline)]
pub use crate::operands::Operands;
#[doc(inline)]
pub use crate::polarity::{PolarVal, Polarity};

#[cfg(feature = "derive")]
#[doc(inline)]
pub use crate::Command;
