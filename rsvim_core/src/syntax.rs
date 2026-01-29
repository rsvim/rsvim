//! Tree-sitter based syntax engine.

use crate::prelude::*;
use tree_sitter::Language;
use tree_sitter::Parser;

#[derive(Debug)]
pub struct Syntax {
  parser: Parser,
}

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  Hash,
  strum_macros::Display,
  strum_macros::EnumString,
)]
pub enum LanguageName {
  #[strum(serialize = "rust")]
  Rust,
}

#[derive(Debug)]
pub struct SyntaxManager {
  languages: FoldMap<LanguageName, Language>,
}

impl Default for SyntaxManager {
  fn default() -> Self {
    Self::new()
  }
}

impl SyntaxManager {
  pub fn new() -> Self {
    Self {
      languages: FoldMap::new(),
    }
  }

  pub fn new_syntax(&mut self, lang: LanguageName) -> Syntax {}
}
