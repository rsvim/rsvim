//! Tree-sitter based syntax engine.

pub mod lang;

use tree_sitter::Parser;

pub struct Syntax {
  parser: Parser,
}

pub struct SyntaxManager {}
