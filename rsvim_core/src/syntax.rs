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
use tree_sitter::QueryCursor;
use tree_sitter::StreamingIterator;
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
pub type SyntaxQueryArc = Arc<Query>;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Line (row) index and byte (column) index (2D)
pub struct SyntaxQueryCaptureKey(usize, usize);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SyntaxQueryCaptureValue {
  pub index: u32,
  pub range: tree_sitter::Range,
}

#[derive(Debug)]
pub struct SyntaxQueryCapture {
  start_nodes: FoldMap<SyntaxQueryCaptureKey, SyntaxQueryCaptureValue>,
  end_nodes: FoldMap<SyntaxQueryCaptureKey, SyntaxQueryCaptureValue>,
}

arc_ptr!(SyntaxQueryCapture);

impl SyntaxQueryCapture {
  pub fn start_nodes(
    &self,
  ) -> &FoldMap<SyntaxQueryCaptureKey, SyntaxQueryCaptureValue> {
    &self.start_nodes
  }

  pub fn end_nodes(
    &self,
  ) -> &FoldMap<SyntaxQueryCaptureKey, SyntaxQueryCaptureValue> {
    &self.end_nodes
  }
}

// SyntaxId starts from 1.
structural_id_impl!(usize, SyntaxId, 1);

/// Buffer syntax.
pub struct Syntax {
  id: SyntaxId,

  // Highlight query
  highlight_query: Option<SyntaxQueryArc>,
  highlight_capture: Option<SyntaxQueryCaptureArc>,

  // Parsed syntax tree
  tree: Option<Tree>,

  // Buffer's editing version of the syntax tree, this is copied from the
  // buffer's `editing_version` when starts parsing the buffer.
  editing_version: isize,

  // Syntax parser
  parser: SyntaxParserArc,

  // Filetype, i.e. language name
  filetype: Option<CompactString>,

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
      .field(
        "tree",
        if self.tree.is_some() {
          &"some"
        } else {
          &"none"
        },
      )
      .field("editing_version", &self.editing_version)
      .field("filetype", &self.filetype)
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
    let filetype = lang.name().map(|name| name.to_compact_string());
    let mut parser = Parser::new();
    parser.set_language(lang)?;
    let parser = Arc::new(Mutex::new(parser));
    let highlight_query = match highlight_query {
      Some(source) => Query::new(lang, source)
        .map(|q| Some(Arc::new(q)))
        .unwrap_or(None),
      None => None,
    };

    Ok(Self {
      id: SyntaxId::next(),
      highlight_query,
      highlight_capture: None,
      tree: None,
      editing_version: INVALID_EDITING_VERSION,
      parser,
      filetype,
      pending: vec![],
      parsing: false,
    })
  }

  pub fn id(&self) -> SyntaxId {
    self.id
  }

  pub fn filetype(&self) -> &Option<CompactString> {
    &self.filetype
  }

  pub fn highlight_query(&self) -> Option<SyntaxQueryArc> {
    self.highlight_query.clone()
  }

  pub fn highlight_capture(&self) -> &Option<SyntaxQueryCaptureArc> {
    &self.highlight_capture
  }

  pub fn set_highlight_capture(
    &mut self,
    value: Option<SyntaxQueryCaptureArc>,
  ) {
    self.highlight_capture = value;
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

pub fn parse(
  parser: Arc<Mutex<Parser>>,
  old_tree: Option<Tree>,
  pending_edits: Vec<SyntaxEdit>,
) -> (Option<Tree>, isize, Option<String>) {
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

  let mut last_payload: Option<String> = None;

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
        last_payload = Some(payload);
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
    last_payload = Some(payload);
  }

  (tree, editing_version, last_payload)
}

/// Here is a really trade-off, I mean we have two solutions when querying
/// syntax highlight colors for a buffer:
///
/// 1. Execute the `QueryCursor::matches` on each viewport/window on every
///    TUI frame.
///    - Pros: The viewport is only a very small part compared with the
///      whole buffer, it gives us smaller problem scale and shorter
///      response time.
///    - Cons: The `QueryCursor::matches` needs to pass the buffer text
///      payload as a `&[u8]` (i.e. `&str`) type. But we are using `Rope` as
///      our text backend, which means we will have to convert (part of) the
///      text to a `String` via something like `to_string` API, which leads
///      to massive memory allocation on each frame.
/// 2. Execute the `QueryCursor::matches` on the whole buffer when syntax
///    parser just finishes its parsing.
///    - Pros: Both syntax parsing and highlight querying are done in
///      background job, the CPU workload of TUI rendering on every frame is
///      reduced, it gives us better performance.
///    - Cons: The `QueryCursor::matches` runs longer because the problem
///      scale becomes larger, since we are querying the whole buffer,
///      instead of a window/viewport. This leads to longer response time,
///      i.e. for a very big buffer, user will wait longer time to get the
///      latest highlights after some editings.
pub fn query(
  tree: &Option<Tree>,
  text_payload: &Option<String>,
  highlight_query: &Option<SyntaxQueryArc>,
) -> Option<SyntaxQueryCaptureArc> {
  let mut query_cursor = QueryCursor::new();
  if let Some(syn_tree) = tree
    && let Some(syn_highlight_query) = highlight_query
    && let Some(text_payload) = text_payload
  {
    query_cursor.set_byte_range(0..usize::MAX);
    let mut matches = query_cursor.matches(
      syn_highlight_query,
      syn_tree.root_node(),
      text_payload.as_bytes(),
    );
    let mut start_nodes: FoldMap<
      SyntaxQueryCaptureKey,
      SyntaxQueryCaptureValue,
    > = FoldMap::new();
    let mut end_nodes: FoldMap<SyntaxQueryCaptureKey, SyntaxQueryCaptureValue> =
      FoldMap::new();
    while let Some(mat) = matches.next() {
      for cap in mat.captures {
        let index = cap.index;
        let range = cap.node.range();
        trace!("Captured highlight {}:{:?}", index, range);
        let start_key = SyntaxQueryCaptureKey(
          range.start_point.row,
          range.start_point.column,
        );
        let end_key =
          SyntaxQueryCaptureKey(range.end_point.row, range.end_point.column);
        debug_assert!(!start_nodes.contains_key(&start_key));
        debug_assert!(!end_nodes.contains_key(&end_key));
        start_nodes.insert(start_key, SyntaxQueryCaptureValue { index, range });
        end_nodes.insert(end_key, SyntaxQueryCaptureValue { index, range });
      }
    }
    Some(SyntaxQueryCapture::to_arc(SyntaxQueryCapture {
      start_nodes,
      end_nodes,
    }))
  } else {
    None
  }
}

pub async fn parse_and_query(
  parser: Arc<Mutex<Parser>>,
  old_tree: Option<Tree>,
  highlight_query: Option<SyntaxQueryArc>,
  pending_edits: Vec<SyntaxEdit>,
) -> (Option<Tree>, isize, Option<SyntaxQueryCaptureArc>) {
  let (tree, editing_version, text_payload) =
    parse(parser, old_tree, pending_edits);
  let highlight_capture = query(&tree, &text_payload, &highlight_query);
  (tree, editing_version, highlight_capture)
}
