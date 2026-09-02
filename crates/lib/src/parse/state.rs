//! What one scan has accumulated so far

use crate::def::{Flag, FlagKind};
use crate::error::Error;
use crate::style::Style;

use super::config::ScanConfig;
use super::{Parsed, Scan, Spelling};

/// The scan under construction plus the bookkeeping repeats and conflicts need
pub struct State<'a> {
    scan: Scan<'a>,
    seen: Vec<bool>,
    exclusive: Option<(u16, usize)>,
}

impl<'a> State<'a> {
    pub fn new(flags: usize) -> Self {
        Self {
            scan: Scan::default(),
            seen: vec![false; flags],
            exclusive: None,
        }
    }

    /// Record one flag occurrence, refusing a repeat or a conflict the style forbids
    pub fn record(
        &mut self,
        config: &ScanConfig<'a>,
        position: usize,
        flag: &Flag,
        parsed: Parsed<'a>,
        spelling: Spelling<'a>,
    ) -> Result<(), Error> {
        self.reject_repeat(config.style, position, flag, spelling)?;
        self.reject_conflict(config, position, flag, spelling)?;
        if !flag.is_implemented() {
            self.scan.unimplemented.push(spelling);
        }
        if flag.flag_kind() != FlagKind::Noop {
            self.scan.flags.push(parsed);
        }
        Ok(())
    }

    fn reject_repeat(
        &mut self,
        style: Style,
        position: usize,
        flag: &Flag,
        spelling: Spelling<'a>,
    ) -> Result<(), Error> {
        let Some(seen) = self.seen.get_mut(position) else {
            return Ok(());
        };
        if style == Style::Gnu && !flag.is_repeatable() && *seen {
            return Err(Error::RepeatedFlag(spelling.to_string()));
        }
        *seen = true;
        Ok(())
    }

    fn reject_conflict(
        &mut self,
        config: &ScanConfig<'a>,
        position: usize,
        flag: &Flag,
        spelling: Spelling<'a>,
    ) -> Result<(), Error> {
        let Some(group) = config.policy.exclusive_group(flag.id()) else {
            return Ok(());
        };
        match self.exclusive {
            Some((other, previous)) if other != group => Err(Error::ConflictingFlags {
                current: spelling.to_string(),
                previous: config.preferred_label(previous),
            }),
            _ => {
                self.exclusive = Some((group, position));
                Ok(())
            }
        }
    }

    /// Whether the flag at this position has fired already
    pub fn seen(&self, position: usize) -> bool {
        self.seen.get(position).copied().unwrap_or(false)
    }

    pub fn push_operand(&mut self, arg: &'a str) {
        self.scan.operands.push(arg);
    }

    pub fn extend_operands(&mut self, rest: &[&'a str]) {
        self.scan.operands.extend_from_slice(rest);
    }

    pub const fn operand_count(&self) -> usize {
        self.scan.operands.len()
    }

    /// Keep an unknown long option as an operand when the command is lenient
    pub fn pass_unknown(&mut self, error: Error, arg: &'a str, lenient: bool) -> Result<(), Error> {
        if lenient && matches!(error, Error::UnknownFlag(_)) {
            self.push_operand(arg);
            Ok(())
        } else {
            Err(error)
        }
    }

    pub fn finish(self) -> Scan<'a> {
        self.scan
    }
}
