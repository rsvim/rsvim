//! Tree-sitter based syntax engine.

use crate::prelude::*;
use crate::structural_id_impl;
use compact_str::CompactString;
use compact_str::ToCompactString;
use ropey::Rope;
use std::fmt::Debug;
use tree_sitter::InputEdit;
use tree_sitter::Language;
use tree_sitter::LanguageError;
use tree_sitter::Parser;
use tree_sitter::Tree;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SyntaxStatus {
  Init,
  Parsing,
  NotMatch,
}

#[derive(Debug, Clone)]
pub struct SyntaxEditNew {
  payload: Rope,
  version: usize,
}

impl PartialEq for SyntaxEditNew {
  fn eq(&self, other: &Self) -> bool {
    self.version == other.version
  }
}

impl Eq for SyntaxEditNew {}

impl SyntaxEditNew {
  pub fn new(payload: Rope, version: usize) -> Self {
    Self { payload, version }
  }
}

#[derive(Debug, Clone)]
pub struct SyntaxEditUpdate {
  payload: Rope,
  input: InputEdit,
  version: usize,
}

impl PartialEq for SyntaxEditUpdate {
  fn eq(&self, other: &Self) -> bool {
    self.version == other.version
  }
}

impl Eq for SyntaxEditUpdate {}

impl SyntaxEditUpdate {
  pub fn new(payload: Rope, input: InputEdit, version: usize) -> Self {
    Self {
      payload,
      input,
      version,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxEdit {
  New(SyntaxEditNew),
  Update(SyntaxEditUpdate),
}

pub struct Syntax {
  parser: Parser,
  tree: Option<Tree>,
  status: SyntaxStatus,
  pending_edits: Vec<SyntaxEdit>,
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

impl Syntax {
  pub fn new(lang: &Language) -> Result<Self, LanguageError> {
    let mut parser = Parser::new();
    parser.set_language(lang)?;
    Ok(Self {
      parser,
      tree: None,
      status: SyntaxStatus::Init,
    })
  }

  pub fn status(&self) -> SyntaxStatus {
    self.status
  }

  pub fn set_status(&mut self, status: SyntaxStatus) {
    self.status = status;
  }
}

structural_id_impl!(str, LanguageId);

pub struct SyntaxManager {
  languages: FoldMap<LanguageId, Language>,
  // Maps language ID to file extensions
  id2ext: FoldMap<LanguageId, FoldSet<CompactString>>,
  // Maps file extension to language ID
  ext2id: FoldMap<CompactString, LanguageId>,
}

impl Debug for SyntaxManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SyntaxManager")
      .field("languages", &self.languages)
      .field("id2ext", &self.id2ext)
      .field("ext2id", &self.ext2id)
      .finish()
  }
}

impl SyntaxManager {
  pub fn new() -> Self {
    let mut it = Self {
      languages: FoldMap::new(),
      id2ext: FoldMap::new(),
      ext2id: FoldMap::new(),
    };
    let rust_id = LanguageId::from("rust");
    it.languages
      .insert(rust_id.clone(), tree_sitter_rust::LANGUAGE.into());
    it.insert_file_ext(rust_id, "rs");
    it
  }

  /// Associate a language ID with a file extension.
  ///
  /// For example, a 'C++' language ID can be associate with below file
  /// extensions:
  /// - Feader files: hh, h++, hpp
  /// - Source files: cc, c++, cpp
  pub fn insert_file_ext(&mut self, id: LanguageId, ext: &str) {
    self
      .id2ext
      .entry(id.clone())
      .or_default()
      .insert(ext.to_compact_string());
    self.ext2id.entry(ext.to_compact_string()).or_insert(id);
  }

  /// Un-associate a language ID with a file extension.
  pub fn remove_file_ext(&mut self, id: LanguageId, ext: &str) {
    self.id2ext.entry(id).or_default().remove(ext);
    self.ext2id.remove(ext);
  }

  pub fn get_file_ext_by_id(
    &self,
    id: &LanguageId,
  ) -> Option<&FoldSet<CompactString>> {
    self.id2ext.get(id)
  }

  pub fn get_id_by_file_ext(&self, ext: &str) -> Option<&LanguageId> {
    self.ext2id.get(ext)
  }

  pub fn insert_lang(&mut self, id: LanguageId, lang: Language) {
    self.languages.insert(id.clone(), lang);
    self.id2ext.entry(id.clone()).or_default();
  }

  pub fn get_lang(&self, id: LanguageId) -> Option<&Language> {
    self.languages.get(&id)
  }

  pub fn get_lang_by_ext(&self, ext: &str) -> Option<&Language> {
    match self.ext2id.get(ext) {
      Some(id) => self.get_lang(id.clone()),
      None => None,
    }
  }
}

impl Default for SyntaxManager {
  fn default() -> Self {
    Self::new()
  }
}
