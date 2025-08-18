//! Vim ex command parser.

use compact_str::CompactString;

pub struct Parser {
  name: CompactString,
  source: CompactString,
}

impl Parser {
  pub fn from_compact_str(name: CompactString, source: CompactString) -> Self {
    Self { name, source }
  }
}
