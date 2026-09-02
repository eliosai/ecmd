//! What one scan has accumulated so far

use crate::error::Error;
use crate::meta::Storage;
use crate::style::Style;

use super::config::ScanConfig;
use super::{FlagDef, FlagKind, OnUnknown, Parsed, Scan, Spelling};

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
    pub fn record<S: Storage, P: Storage>(
        &mut self,
        config: &ScanConfig<'a, S, P>,
        position: usize,
        def: &FlagDef<S>,
        parsed: Parsed<'a>,
        spelling: Spelling<'a>,
    ) -> Result<(), Error> {
        self.reject_repeat(config.style, position, def, spelling)?;
        self.reject_conflict(config, position, def, spelling)?;
        if !def.implemented {
            self.scan.unimplemented.push(spelling);
        }
        if def.kind != FlagKind::Noop {
            self.scan.flags.push(parsed);
        }
        Ok(())
    }

    fn reject_repeat<S: Storage>(
        &mut self,
        style: Style,
        position: usize,
        def: &FlagDef<S>,
        spelling: Spelling<'a>,
    ) -> Result<(), Error> {
        let Some(seen) = self.seen.get_mut(position) else {
            return Ok(());
        };
        if style == Style::Gnu && !def.repeatable && *seen {
            return Err(Error::RepeatedFlag(spelling.to_string()));
        }
        *seen = true;
        Ok(())
    }

    fn reject_conflict<S: Storage, P: Storage>(
        &mut self,
        config: &ScanConfig<'a, S, P>,
        position: usize,
        def: &FlagDef<S>,
        spelling: Spelling<'a>,
    ) -> Result<(), Error> {
        let Some(group) = config.policy.exclusive_group(def.ch) else {
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

    /// Keep an unknown long option as an operand when the command passes unknowns through
    pub fn pass_unknown(
        &mut self,
        error: Error,
        arg: &'a str,
        on_unknown: OnUnknown,
    ) -> Result<(), Error> {
        if on_unknown == OnUnknown::PassThrough && matches!(error, Error::UnknownFlag(_)) {
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
