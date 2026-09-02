//! What every scanning step shares

use crate::def::{Def, Flag, Policy};
use crate::style::Style;

use super::classify::is_polar;

/// The declared flags, the policy and the style one scan runs under
pub struct ScanConfig<'a> {
    pub flags: &'a [Flag],
    pub policy: &'a Policy,
    pub lenient: bool,
    pub permute: bool,
    pub style: Style,
    pub has_polarity: bool,
    pub index: [u16; 128],
}

impl<'a> ScanConfig<'a> {
    pub fn new(def: &'a Def) -> Self {
        let flags = def.flags();
        let mut index = [0_u16; 128];
        for (position, flag) in flags.iter().enumerate().rev() {
            if let Some(slot) = ascii_slot(flag.id()).and_then(|slot| index.get_mut(slot))
                && let Ok(entry) = u16::try_from(position.saturating_add(1))
            {
                *slot = entry;
            }
        }
        Self {
            flags,
            policy: def.policy(),
            lenient: def.lenient(),
            permute: def.permute(),
            style: def.style(),
            has_polarity: flags.iter().any(|flag| is_polar(flag.flag_kind())),
            index,
        }
    }

    pub fn gnu(&self) -> bool {
        self.style == Style::Gnu
    }

    /// The position and definition of the flag with this identity
    pub fn find(&self, id: char) -> Option<(usize, &'a Flag)> {
        let indexed = ascii_slot(id)
            .and_then(|slot| self.index.get(slot))
            .map_or(0, |entry| usize::from(*entry));
        if indexed > 0 {
            let position = indexed.saturating_sub(1);
            return self.flags.get(position).map(|flag| (position, flag));
        }
        self.flags
            .iter()
            .enumerate()
            .find(|(_, flag)| flag.id() == id)
    }

    /// The label the command prefers for a flag: its short, or its long when it has no short
    pub fn preferred_label(&self, position: usize) -> String {
        self.flags.get(position).map_or_else(String::new, |flag| {
            flag.short().map_or_else(
                || format!("--{}", flag.long_name().unwrap_or("")),
                |short| format!("-{short}"),
            )
        })
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
