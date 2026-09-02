//! The position in one argument slice

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

    /// The argument under the cursor, moving past it
    pub fn take(&mut self) -> Option<&'a str> {
        let value = self.peek()?;
        self.advance();
        Some(value)
    }

    /// The argument under the cursor when it passes the test, moving past it
    pub fn take_if(&mut self, accept: impl FnOnce(&str) -> bool) -> Option<&'a str> {
        let value = self.peek().filter(|value| accept(value))?;
        self.advance();
        Some(value)
    }

    pub fn rest(&self) -> &'a [&'a str] {
        self.args.get(self.pos..).unwrap_or_default()
    }
}
