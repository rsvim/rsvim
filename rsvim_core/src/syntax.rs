//! Tree-sitter based syntax parser.

use tree_sitter::InputEdit;
use tree_sitter::Language;
use tree_sitter::Parser;
use tree_sitter::Point;

pub struct Syntax {
  parser: Parser,
}

impl Syntax {
  pub fn new() -> Self {
    Self {
      parser: Parser::new(),
    }
  }
}
