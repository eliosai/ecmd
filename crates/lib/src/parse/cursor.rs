//! The position in one argument slice

use crate::error::Error;

pub struct Cursor<'a> {
    args: &'a [&'a str],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub const fn new(args: &'a [&'a str]) -> Self {
        Self { args, pos: 0 }
    }

    pub fn peek(&self) -> Option<&'a str> {
        self.args.get(self.pos).copied()
    }

    pub const fn at_start(&self) -> bool {
        self.pos == 0
    }

    pub const fn advance(&mut self) {
        self.pos = self.pos.saturating_add(1);
    }

    pub fn next_value(&mut self, flag_ch: char) -> Result<String, Error> {
        self.advance();
        self.take_next(&format!("-{flag_ch}"))
    }

    pub fn take_next(&mut self, opt: &str) -> Result<String, Error> {
        let val = self
            .peek()
            .map(ToOwned::to_owned)
            .ok_or_else(|| Error::MissingValue(opt.to_owned()))?;
        self.advance();
        Ok(val)
    }

    pub fn rest(&self) -> &'a [&'a str] {
        self.args.get(self.pos..).unwrap_or_default()
    }

    pub fn following(&self) -> Option<&'a str> {
        self.args.get(self.pos.saturating_add(1)).copied()
    }
}
