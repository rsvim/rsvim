//! Tree-sitter based syntax engine.

use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
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
  // Maps language ID to file extensions
  id2ext: FoldMap<LanguageId, FoldSet<CompactString>>,
  // Maps file extension to language ID
  ext2id: FoldMap<CompactString, LanguageId>,
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
      id2ext: FoldMap::new(),
      ext2id: FoldMap::new(),
    }
  }

  /// Associate a language ID with a file extension.
  ///
  /// For example, a 'C++' language ID can be associate with below file
  /// extensions:
  /// - Feader files: h, hh, h++, hpp
  /// - Source files: cpp, cc, c++
  pub fn insert_file_ext(&mut self, lang_id: LanguageId, ext: &str) {
    self
      .id2ext
      .entry(lang_id)
      .or_default()
      .insert(ext.to_compact_string());
    self
      .ext2id
      .entry(ext.to_compact_string())
      .or_insert(lang_id);
  }

  /// Un-associate a language ID with a file extension.
  pub fn remove_file_ext(&mut self, lang_id: LanguageId, ext: &str) {
    self.id2ext.entry(lang_id).or_default().remove(ext);
    self.ext2id.remove(ext);
  }

  pub fn get_lang(&mut self, lang_id: LanguageId) -> Option<&Language> {
    self
      .languages
      .entry(lang_id)
      .or_insert_with(|| tree_sitter_rust::LANGUAGE.into());
    self.languages.get(&lang_id)
  }

  pub fn get_lang_by_ext(&mut self, ext: &str) -> Option<&Language> {
    match self.ext2id.get(ext) {
      Some(lang_id) => self.get_lang(*lang_id),
      None => None,
    }
  }
}
