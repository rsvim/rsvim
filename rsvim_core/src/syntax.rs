//! Tree-sitter based syntax engine.

use crate::prelude::*;
use tree_sitter::Language;
use tree_sitter::LanguageError;
use tree_sitter::Parser;
use tree_sitter::Tree;

pub struct Syntax {
  parser: Parser,
  tree: Option<Tree>,
}

impl Syntax {
  pub fn new() -> Self {
    Self {
      parser: Parser::new(),
      tree: None,
    }
  }

  pub fn set_language(&mut self, lang: &Language) -> Result<(), LanguageError> {
    self.parser.set_language(lang)
  }
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

  pub fn new_syntax(&mut self, lang: LanguageName) -> Syntax {
    self
      .languages
      .entry(lang)
      .or_insert_with(|| tree_sitter_rust::LANGUAGE.into());
    let mut syn = Syntax::new();
    syn
      .set_language(self.languages.get(&lang).unwrap())
      .unwrap();
    syn
  }
}
