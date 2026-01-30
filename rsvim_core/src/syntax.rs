//! Tree-sitter based syntax engine.

use crate::prelude::*;
use std::fmt::Debug;
use tree_sitter::Language;
use tree_sitter::LanguageError;
use tree_sitter::Parser;
use tree_sitter::Tree;

pub struct Syntax {
  parser: Parser,
  tree: Option<Tree>,
}

impl Debug for Syntax {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Syntax")
      .field(
        "parser",
        &self
          .parser
          .language()
          .map(|l| l.name().unwrap_or("unknown"))
          .unwrap_or("unknown"),
      )
      .field(
        "tree",
        if self.tree.is_some() {
          &"some"
        } else {
          &"none"
        },
      )
      .finish()
  }
}

impl Default for Syntax {
  fn default() -> Self {
    Self::new()
  }
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
pub enum LanguageId {
  #[strum(serialize = "rust")]
  Rust,
}

#[derive(Debug, Clone)]
pub struct SyntaxManager {
  languages: FoldMap<LanguageId, Language>,
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

  pub fn get_language(&mut self, lang: LanguageId) -> Option<&Language> {
    self
      .languages
      .entry(lang)
      .or_insert_with(|| tree_sitter_rust::LANGUAGE.into());
    self.languages.get(&lang)
  }
}
