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

#[derive(Clone)]
pub struct SyntaxEditNew {
  pub payload: Rope,
  pub version: isize,
}

impl Debug for SyntaxEditNew {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SyntaxEditNew")
      .field(
        "payload",
        &self
          .payload
          .get_line(0)
          .map(|l| l.to_string())
          .unwrap_or("".to_string()),
      )
      .field("version", &self.version)
      .finish()
  }
}

#[derive(Clone)]
pub struct SyntaxEditUpdate {
  pub payload: Rope,
  pub input: InputEdit,
  pub version: isize,
}

impl Debug for SyntaxEditUpdate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SyntaxEditUpdate")
      .field(
        "payload",
        &self
          .payload
          .get_line(0)
          .map(|l| l.to_string())
          .unwrap_or("".to_string()),
      )
      .field("input", &self.input)
      .field("version", &self.version)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub enum SyntaxEdit {
  New(SyntaxEditNew),
  Update(SyntaxEditUpdate),
}

/// Buffer syntax.
pub struct Syntax {
  // Parsed syntax tree
  tree: Option<Tree>,

  // Buffer's editing version of the syntax tree, this is copied from the
  // buffer's `editing_version` when starts parsing the buffer.
  editing_version: isize,

  // Syntax parser
  parser: Parser,

  // Pending edits that waiting for parsing
  pending: Vec<SyntaxEdit>,

  // Whether there's already a background task running on parsing.
  // NOTE: At a certain timing, only 1 background task is running to parse a
  // buffer. New editings will be pushed to the `pending` job queue and wait
  // for the **current** task complete, then starts the next new task.
  parsing: bool,
}

impl Debug for Syntax {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Syntax")
      .field(
        "tree",
        if self.tree.is_some() {
          &"some"
        } else {
          &"none"
        },
      )
      .field("editing_version", &self.editing_version)
      .field(
        "parser",
        &self
          .parser
          .language()
          .map(|l| l.name().unwrap_or("unknown"))
          .unwrap_or("unknown"),
      )
      .field("pending", &self.pending)
      .field("parsing", &self.parsing)
      .finish()
  }
}

const INVALID_EDITING_VERSION: isize = -1;

impl Syntax {
  pub fn new(lang: &Language) -> Result<Self, LanguageError> {
    let mut parser = Parser::new();
    parser.set_language(lang)?;
    Ok(Self {
      tree: None,
      editing_version: INVALID_EDITING_VERSION,
      parser,
      pending: vec![],
      parsing: false,
    })
  }

  pub fn is_parsing(&self) -> bool {
    self.parsing
  }

  pub fn set_is_parsing(&mut self) {
    self.parsing = true;
  }

  pub fn set_not_parsing(&mut self) {
    self.parsing = false;
  }

  pub fn editing_version(&self) -> isize {
    self.editing_version
  }

  pub fn set_editing_version(&mut self, value: isize) {
    self.editing_version = value;
  }

  pub fn add_pending_edit(&mut self, value: SyntaxEdit) {
    self.pending.push(value);
  }

  pub fn drain_pending_edit<R>(
    &mut self,
    range: R,
  ) -> std::vec::Drain<'_, SyntaxEdit>
  where
    R: std::ops::RangeBounds<usize>,
  {
    self.pending.drain(range)
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
