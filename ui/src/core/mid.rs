use std::{fmt::Display, write};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Mid(usize);
impl Display for Mid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct MidGenerator(usize);

impl MidGenerator {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn next(&mut self) -> Mid {
        self.0 += 1;
        Mid(self.0)
    }
}
