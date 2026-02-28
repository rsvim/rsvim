#![allow(dead_code)]
//! Tree-sitter based syntax engine.

use crate::buf::Buffer;
use crate::prelude::*;
use crate::structural_id_impl;
use compact_str::CompactString;
use compact_str::ToCompactString;
use parking_lot::Mutex;
use ropey::Rope;
use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;
use tree_sitter::InputEdit;
use tree_sitter::Language;
use tree_sitter::LanguageError;
use tree_sitter::Parser;
use tree_sitter::Point;
use tree_sitter::Query;
use tree_sitter::Tree;

const INVALID_EDITING_VERSION: isize = -1;

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

pub type SyntaxParserArc = std::sync::Arc<parking_lot::Mutex<Parser>>;
pub type SyntaxParserWk = std::sync::Weak<parking_lot::Mutex<Parser>>;
pub type SyntaxParserMutexGuard<'a> = parking_lot::MutexGuard<'a, Parser>;

// SyntaxId starts from 1
structural_id_impl!(usize, SyntaxId, 1);

/// Buffer syntax.
pub struct Syntax {
  // ID
  id: SyntaxId,

  // Highlight query
  highlight_query: Option<Query>,

  // Parsed syntax tree
  tree: Option<Tree>,

  // Buffer's editing version of the syntax tree, this is copied from the
  // buffer's `editing_version` when starts parsing the buffer.
  editing_version: isize,

  // Syntax parser
  parser: SyntaxParserArc,

  // Language (filetype)
  language: Option<CompactString>,

  // Pending edits that waiting for parsing
  pending: Vec<SyntaxEdit>,

  // Whether the parser is already parsing the buffer text in a background
  // task. If true, it means the `parser` is parsing in a background task.
  //
  // NOTE: At a certain time, only 1 background task is parsing a buffer, there
  // will be no multiple background tasks parsing the same buffer
  // simultaneously, for data safety reason. New editings will be add to the
  // `pending` job queue and wait for the **current** running task complete,
  // then starts the next new task.
  parsing: bool,
}

impl Debug for Syntax {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Syntax")
      .field("id", &self.id)
      .field(
        "tree",
        if self.tree.is_some() {
          &"some"
        } else {
          &"none"
        },
      )
      .field("editing_version", &self.editing_version)
      .field("language", &self.language)
      .field("pending", &self.pending)
      .field("parsing", &self.parsing)
      .finish()
  }
}

impl Syntax {
  pub fn new(
    lang: &Language,
    highlight_query: Option<&String>,
  ) -> Result<Self, LanguageError> {
    let language = lang.name().map(|name| name.to_compact_string());
    let mut parser = Parser::new();
    parser.set_language(lang)?;
    let parser = Arc::new(Mutex::new(parser));
    let highlight_query = match highlight_query {
      Some(source) => Query::new(lang, source).map(Some).unwrap_or(None),
      None => None,
    };

    Ok(Self {
      id: SyntaxId::next(),
      highlight_query,
      tree: None,
      editing_version: INVALID_EDITING_VERSION,
      parser,
      language,
      pending: vec![],
      parsing: false,
    })
  }

  pub fn id(&self) -> SyntaxId {
    self.id
  }

  pub fn tree(&self) -> &Option<Tree> {
    &self.tree
  }

  pub fn set_tree(&mut self, tree: Option<Tree>) {
    self.tree = tree;
  }

  pub fn editing_version(&self) -> isize {
    self.editing_version
  }

  pub fn set_editing_version(&mut self, value: isize) {
    self.editing_version = value;
  }

  pub fn parser(&self) -> SyntaxParserArc {
    self.parser.clone()
  }

  pub fn is_parsing(&self) -> bool {
    self.parsing
  }

  pub fn set_is_parsing(&mut self, value: bool) {
    self.parsing = value;
  }

  pub fn pending_is_empty(&self) -> bool {
    self.pending.is_empty()
  }

  pub fn pending_len(&self) -> usize {
    self.pending.len()
  }

  pub fn add_pending(&mut self, value: SyntaxEdit) {
    self.pending.push(value);
  }

  pub fn drain_pending<R>(
    &mut self,
    range: R,
  ) -> std::vec::Drain<'_, SyntaxEdit>
  where
    R: std::ops::RangeBounds<usize>,
  {
    self.pending.drain(range)
  }
}

pub struct SyntaxManager {
  languages: FoldMap<CompactString, Language>,
  highlight_queries: FoldMap<CompactString, String>,

  // Maps language ID to file extensions
  id2ext: FoldMap<CompactString, FoldSet<CompactString>>,
  // Maps file extension to language ID
  ext2id: FoldMap<CompactString, CompactString>,
}

impl Debug for SyntaxManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SyntaxManager")
      .field("languages", &self.languages)
      .field("highlight_queries", &self.highlight_queries)
      .field("id2ext", &self.id2ext)
      .field("ext2id", &self.ext2id)
      .finish()
  }
}

// Language ID and file extensions {
impl SyntaxManager {
  pub fn new() -> Self {
    let mut it = Self {
      languages: FoldMap::new(),
      highlight_queries: FoldMap::new(),
      id2ext: FoldMap::new(),
      ext2id: FoldMap::new(),
    };

    let language_bindings = [
      (
        "rust",
        tree_sitter_rust::LANGUAGE,
        Some(tree_sitter_rust::HIGHLIGHTS_QUERY),
        vec!["rs"],
      ),
      (
        "markdown",
        tree_sitter_md::LANGUAGE,
        Some(tree_sitter_md::HIGHLIGHT_QUERY_BLOCK),
        vec!["md", "markdown"],
      ),
      (
        "toml",
        tree_sitter_toml_ng::LANGUAGE,
        Some(tree_sitter_toml_ng::HIGHLIGHTS_QUERY),
        vec!["toml"],
      ),
    ];

    for lang_binding in language_bindings {
      for lang_ext in lang_binding.3.iter() {
        it.insert_file_ext(
          lang_binding.0.to_compact_string(),
          lang_ext.to_compact_string(),
        );
      }
      it.insert_lang(
        lang_binding.0.to_compact_string(),
        lang_binding.1.into(),
        lang_binding.2.map(|q| q.to_string()),
      );
    }

    it
  }

  /// Associate a language ID with a file extension.
  ///
  /// For example, a 'C++' language ID can be associate with below file
  /// extensions:
  /// - Feader files: hh, h++, hpp
  /// - Source files: cc, c++, cpp
  pub fn insert_file_ext(&mut self, id: CompactString, ext: CompactString) {
    self
      .id2ext
      .entry(id.clone())
      .or_default()
      .insert(ext.clone());
    self.ext2id.entry(ext).or_insert(id);
  }

  /// Un-associate a language ID with a file extension.
  pub fn remove_file_ext(&mut self, id: &str, ext: &str) {
    self
      .id2ext
      .entry(id.to_compact_string())
      .or_default()
      .remove(ext);
    self.ext2id.remove(ext);
  }

  pub fn get_file_ext_by_id(
    &self,
    id: &str,
  ) -> Option<&FoldSet<CompactString>> {
    self.id2ext.get(id)
  }

  pub fn get_id_by_file_ext(&self, ext: &str) -> Option<&CompactString> {
    self.ext2id.get(ext)
  }
}
// Language ID and file extensions }

// Language and queries {
impl SyntaxManager {
  pub fn insert_lang(
    &mut self,
    id: CompactString,
    lang: Language,
    highlight_query: Option<String>,
  ) {
    self.languages.insert(id.clone(), lang);
    if let Some(hl_query) = highlight_query {
      self.highlight_queries.insert(id.clone(), hl_query);
    }
    self.id2ext.entry(id.clone()).or_default();
  }

  pub fn get_lang(&self, id: &str) -> Option<&Language> {
    self.languages.get(id)
  }

  pub fn get_highlight_query(&self, id: &str) -> Option<&String> {
    self.highlight_queries.get(id)
  }

  pub fn get_lang_by_ext(&self, ext: &str) -> Option<&Language> {
    self
      .ext2id
      .get(ext)
      .map(|id| self.get_lang(id))
      .unwrap_or(None)
  }

  pub fn get_highlight_query_by_ext(&self, ext: &str) -> Option<&String> {
    self
      .ext2id
      .get(ext)
      .map(|id| self.get_highlight_query(id))
      .unwrap_or(None)
  }
}
// Language and queries }

impl Default for SyntaxManager {
  fn default() -> Self {
    Self::new()
  }
}

fn convert_edit_char_to_byte(rope: &Rope, absolute_char_idx: usize) -> usize {
  rope
    .try_char_to_byte(absolute_char_idx)
    .unwrap_or(rope.len_bytes())
}

fn convert_edit_char_to_point(rope: &Rope, absolute_char_idx: usize) -> Point {
  if absolute_char_idx >= rope.len_chars() {
    let row = rope.len_lines();
    let column = 0;
    tree_sitter::Point { row, column }
  } else {
    let row = rope.char_to_line(absolute_char_idx);
    debug_assert!(rope.get_line(row).is_some());
    let relative_char_idx = absolute_char_idx - rope.line_to_char(row);
    let line = rope.line(row);
    let column = line
      .try_char_to_byte(relative_char_idx)
      .unwrap_or(line.len_bytes());
    tree_sitter::Point { row, column }
  }
}

pub fn make_input_edit_by_delete(
  buffer: &Buffer,
  absolute_char_idx_range: &Range<usize>,
) -> Option<InputEdit> {
  if buffer.syntax().is_some() {
    let start_byte = convert_edit_char_to_byte(
      buffer.text().rope(),
      absolute_char_idx_range.start,
    );
    let old_end_byte = convert_edit_char_to_byte(
      buffer.text().rope(),
      absolute_char_idx_range.end,
    );
    let new_end_byte = start_byte;
    let start_position = convert_edit_char_to_point(
      buffer.text().rope(),
      absolute_char_idx_range.start,
    );
    let old_end_position = convert_edit_char_to_point(
      buffer.text().rope(),
      absolute_char_idx_range.end,
    );
    let new_end_position = start_position;

    Some(InputEdit {
      start_byte,
      old_end_byte,
      new_end_byte,
      start_position,
      old_end_position,
      new_end_position,
    })
  } else {
    None
  }
}

pub fn make_input_edit_by_insert(
  buffer: &Buffer,
  absolute_char_idx: usize,
  absolute_end_char_idx: usize,
) -> Option<InputEdit> {
  if buffer.syntax().is_some() {
    let start_byte =
      convert_edit_char_to_byte(buffer.text().rope(), absolute_char_idx);
    let old_end_byte = start_byte;
    let new_end_byte =
      convert_edit_char_to_byte(buffer.text().rope(), absolute_end_char_idx);
    let start_position =
      convert_edit_char_to_point(buffer.text().rope(), absolute_char_idx);
    let old_end_position = start_position;
    let new_end_position =
      convert_edit_char_to_point(buffer.text().rope(), absolute_end_char_idx);
    Some(InputEdit {
      start_byte,
      old_end_byte,
      new_end_byte,
      start_position,
      old_end_position,
      new_end_position,
    })
  } else {
    None
  }
}

pub async fn parse(
  parser: Arc<Mutex<Parser>>,
  old_tree: Option<Tree>,
  pending_edits: Vec<SyntaxEdit>,
) -> (Option<Tree>, isize) {
  let mut parser = lock!(parser);
  let mut tree = old_tree;
  let mut editing_version = INVALID_EDITING_VERSION;

  if cfg!(debug_assertions) {
    let mut new_count: usize = 0;
    for (i, edit) in pending_edits.iter().enumerate() {
      match edit {
        SyntaxEdit::New(_) => {
          debug_assert_eq!(i, 0);
          debug_assert_eq!(new_count, 0);
          new_count += 1;
          debug_assert_eq!(new_count, 1);
        }
        SyntaxEdit::Update(_) => {}
      }
    }
    debug_assert!(new_count <= 1);
  }

  if !pending_edits.is_empty() && matches!(pending_edits[0], SyntaxEdit::New(_))
  {
    match &pending_edits[0] {
      SyntaxEdit::New(new) => {
        let payload = new.payload.to_string();
        let new_tree = parser.parse(&payload, tree.as_ref());
        tree = new_tree;
        editing_version = new.version;
        trace!(
          "Parsed new tree:{:?}, editing_version:{:?}",
          tree
            .clone()
            .map(|t| t.root_node().to_string())
            .unwrap_or("None".to_string()),
          editing_version
        );
      }
      _ => unreachable!(),
    }
  }

  let mut last_update: Option<&SyntaxEditUpdate> = None;
  for (i, edit) in pending_edits.iter().enumerate() {
    if matches!(edit, SyntaxEdit::New(_)) {
      debug_assert_eq!(i, 0);
      continue;
    }
    match edit {
      SyntaxEdit::Update(update) => {
        debug_assert!(tree.is_some());
        if let Some(ref mut tree1) = tree {
          tree1.edit(&update.input);
        }
        last_update = Some(update);
      }
      SyntaxEdit::New(_) => unreachable!(),
    }
  }

  if let Some(last_update) = last_update {
    let payload = last_update.payload.to_string();
    let new_tree = parser.parse(&payload, tree.as_ref());
    tree = new_tree;
    editing_version = last_update.version;
    trace!(
      "Parsed update tree:{:?}, editing_version:{:?}",
      tree
        .clone()
        .map(|t| t.root_node().to_string())
        .unwrap_or("None".to_string()),
      editing_version
    );
  }

  (tree, editing_version)
}
