//! Tree-sitter based syntax engine.

pub mod lang;

use crate::prelude::*;
pub use lang::*;
use tree_sitter::Language;
use tree_sitter::Parser;

pub struct Syntax {
  parser: Parser,
}

pub struct SyntaxManager {
  languages: FoldMap<LanguageName, Language>,
}
