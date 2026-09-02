//! What every scanning step shares

use crate::meta::Storage;
use crate::style::Style;

use super::classify::is_polar;
use super::{FlagDef, OnUnknown, Policy};

/// The declared flags, the policy and the style one scan runs under
pub struct ScanConfig<'a, S: Storage, P: Storage> {
    pub flags: &'a [FlagDef<S>],
    pub policy: Policy<'a, P>,
    pub on_unknown: OnUnknown,
    pub style: Style,
    pub has_polarity: bool,
    pub index: [u16; 128],
}

impl<'a, S: Storage, P: Storage> ScanConfig<'a, S, P> {
    pub fn new(
        flags: &'a [FlagDef<S>],
        policy: Policy<'a, P>,
        on_unknown: OnUnknown,
        style: Style,
    ) -> Self {
        let mut index = [0_u16; 128];
        for (position, flag) in flags.iter().enumerate().rev() {
            if let Some(slot) = ascii_slot(flag.ch).and_then(|slot| index.get_mut(slot))
                && let Ok(entry) = u16::try_from(position.saturating_add(1))
            {
                *slot = entry;
            }
        }
        Self {
            flags,
            policy,
            on_unknown,
            style,
            has_polarity: flags.iter().any(|flag| is_polar(&flag.kind)),
            index,
        }
    }

    pub fn gnu(&self) -> bool {
        self.style == Style::Gnu
    }

    /// The position and definition of the flag with this identity
    pub fn find(&self, ch: char) -> Option<(usize, &'a FlagDef<S>)> {
        let indexed = ascii_slot(ch)
            .and_then(|slot| self.index.get(slot))
            .map_or(0, |entry| usize::from(*entry));
        if indexed > 0 {
            let position = indexed.saturating_sub(1);
            return self.flags.get(position).map(|flag| (position, flag));
        }
        self.flags
            .iter()
            .enumerate()
            .find(|(_, flag)| flag.ch == ch)
    }

    /// The label the command prefers for a flag: its short, or its long when it has no short
    pub fn preferred_label(&self, position: usize) -> String {
        match self.flags.get(position) {
            Some(flag) if flag.ch >= '\u{E000}' => format!("--{}", flag.long.as_ref()),
            Some(flag) => format!("-{}", flag.ch),
            None => String::new(),
        }
    }
}

/// The index slot for an ASCII identity, and none for a synthetic or wide one
fn ascii_slot(ch: char) -> Option<usize> {
    u8::try_from(ch).ok().filter(u8::is_ascii).map(usize::from)
}

/// A `--name` or `--name=value` token, but not the bare `--` terminator
pub fn is_long(arg: &str) -> bool {
    arg.len() > 2 && arg.starts_with("--")
}
