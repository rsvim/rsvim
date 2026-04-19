//! Tree-sitter based syntax engine.

use crate::buf::Buffer;
use crate::prelude::*;
use crate::structural_id_impl;
use compact_str::CompactString;
use compact_str::ToCompactString;
use itertools::Itertools;
use itertools::process_results;
use ropey::Rope;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::Weak;
use tree_sitter::InputEdit;
use tree_sitter::Language;
use tree_sitter::LanguageError;
use tree_sitter::Parser;
use tree_sitter::Point;
use tree_sitter::Query;
use tree_sitter::QueryCursor;
use tree_sitter::StreamingIterator;
use tree_sitter::Tree;
use tree_sitter_loader::CompileConfig;
use tree_sitter_loader::Loader;

const INVALID_EDITING_VERSION: isize = -1;

pub type TreeSitterParserArc = Arc<Mutex<Parser>>;
pub type TreeSitterParserWk = Weak<Mutex<Parser>>;
pub type TreeSitterParserMutexGuard<'a> = MutexGuard<'a, Parser>;

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

pub type TreeSitterQueryArc = Arc<Query>;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SyntaxCapturePoint {
  pub line_idx: usize,
  pub char_idx: usize,
}

impl Ord for SyntaxCapturePoint {
  fn cmp(&self, other: &Self) -> Ordering {
    match self.line_idx.cmp(&other.line_idx) {
      Ordering::Equal => self.char_idx.cmp(&other.char_idx),
      Ordering::Greater => Ordering::Greater,
      Ordering::Less => Ordering::Less,
    }
  }
}

impl PartialOrd for SyntaxCapturePoint {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Convert [`tree_sitter::Range`] based bytes indexing into [`ropey::Rope`]
/// based chars indexing, i.e. use [`ropey::Rope::byte_to_char`] API to
/// transform byte index to char index.
pub struct SyntaxCaptureRange {
  pub start_char: usize,
  pub end_char: usize,
  pub start_point: SyntaxCapturePoint,
  pub end_point: SyntaxCapturePoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxCaptureValue {
  pub index: u32,
  pub name: CompactString,
  pub range: SyntaxCaptureRange,
}

#[derive(Debug, Clone)]
pub struct SyntaxCaptureMultipleValues {
  pub values: Vec<SyntaxCaptureValue>,
  pub max_end_char: Option<usize>,
  pub max_end_point: Option<SyntaxCapturePoint>,
}

pub type SyntaxCaptureMap =
  FoldMap<SyntaxCapturePoint, SyntaxCaptureMultipleValues>;

#[derive(Debug)]
pub struct SyntaxCapture {
  // Maps start_point to all its captured nodes.
  nodes: SyntaxCaptureMap,
}

arc_ptr!(SyntaxCapture);

impl SyntaxCapture {
  pub fn new(nodes: SyntaxCaptureMap) -> Self {
    Self { nodes }
  }

  pub fn nodes(&self) -> &SyntaxCaptureMap {
    &self.nodes
  }
}

// SyntaxId starts from 1.
structural_id_impl!(usize, SyntaxId, 1);

/// Buffer syntax.
pub struct Syntax {
  id: SyntaxId,

  // Highlight query
  highlight_query: Option<TreeSitterQueryArc>,
  highlight_capture: Option<SyntaxCaptureArc>,

  // Parsed syntax tree
  tree: Option<Tree>,

  // Buffer's editing version of the syntax tree, this is copied from the
  // buffer's `editing_version` when starts parsing the buffer.
  editing_version: isize,

  // Syntax parser
  parser: TreeSitterParserArc,

  // Filetype, i.e. programming language name, grammar id
  filetype: Option<CompactString>,

  // Pending edits that waiting for parsing
  pending_edits: Vec<SyntaxEdit>,

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
      .field("pending_edits", &self.pending_edits)
      .field("parsing", &self.parsing)
      .finish()
  }
}

impl Syntax {
  pub fn new(
    grammar: &Language,
    highlight_query: Option<&String>,
  ) -> Result<Self, LanguageError> {
    let filetype = grammar.name().map(|name| name.to_compact_string());
    let mut parser = Parser::new();
    parser.set_language(grammar)?;
    let parser = Arc::new(Mutex::new(parser));
    let highlight_query = match highlight_query {
      Some(source) => Query::new(grammar, source)
        .map(|q| Some(Arc::new(q)))
        .unwrap_or(None),
      None => None,
    };
    // trace!(
    //   "capture names:{:?}",
    //   highlight_query.as_ref().map(|hq| hq.capture_names())
    // );

    Ok(Self {
      id: SyntaxId::next(),
      highlight_query,
      highlight_capture: None,
      tree: None,
      editing_version: INVALID_EDITING_VERSION,
      parser,
      filetype,
      pending_edits: vec![],
      parsing: false,
    })
  }

  pub fn id(&self) -> SyntaxId {
    self.id
  }

  pub fn filetype(&self) -> &Option<CompactString> {
    &self.filetype
  }

  pub fn treesitter_highlight_query(&self) -> Option<TreeSitterQueryArc> {
    self.highlight_query.clone()
  }

  pub fn highlight_capture(&self) -> Option<SyntaxCaptureArc> {
    self.highlight_capture.clone()
  }

  pub fn set_highlight_capture(&mut self, value: Option<SyntaxCaptureArc>) {
    self.highlight_capture = value;
  }

  pub fn treesitter_tree(&self) -> &Option<Tree> {
    &self.tree
  }

  pub fn set_treesitter_tree(&mut self, tree: Option<Tree>) {
    self.tree = tree;
  }

  pub fn editing_version(&self) -> isize {
    self.editing_version
  }

  pub fn set_editing_version(&mut self, value: isize) {
    self.editing_version = value;
  }

  pub fn treesitter_parser(&self) -> TreeSitterParserArc {
    self.parser.clone()
  }

  pub fn is_parsing(&self) -> bool {
    self.parsing
  }

  pub fn set_is_parsing(&mut self, value: bool) {
    self.parsing = value;
  }

  pub fn pending_edits_is_empty(&self) -> bool {
    self.pending_edits.is_empty()
  }

  pub fn pending_edits_len(&self) -> usize {
    self.pending_edits.len()
  }

  pub fn add_pending_edits(&mut self, value: SyntaxEdit) {
    self.pending_edits.push(value);
  }

  pub fn drain_pending_edits<R>(
    &mut self,
    range: R,
  ) -> std::vec::Drain<'_, SyntaxEdit>
  where
    R: std::ops::RangeBounds<usize>,
  {
    self.pending_edits.drain(range)
  }
}

// pub type TreeSitterLoaderArc = Arc<Mutex<Loader>>;
// pub type TreeSitterLoaderWk = Weak<Mutex<Loader>>;
// pub type TreeSitterLoaderMutexGuard<'a> = MutexGuard<'a, Loader>;

pub struct SyntaxLoader {
  loader: Mutex<Loader>,
}

arc_ptr!(SyntaxLoader);

#[derive(Debug, Clone)]
pub struct SyntaxLoadGrammarRequest {
  pub grammar_path: PathBuf,
}

impl SyntaxLoader {
  #[cfg(test)]
  pub fn new() -> Self {
    let loader = Loader::new().unwrap();
    Self {
      loader: Mutex::new(loader),
    }
  }

  #[cfg(not(test))]
  pub fn new() -> Self {
    use crate::cfg::path_cfg::PATH_CONFIG;

    let parser_lib_path =
      PATH_CONFIG.config_home().join(".tree-sitter-parsers");
    Self {
      loader: Mutex::new(Loader::with_parser_lib_path(parser_lib_path.clone())),
    }
  }

  pub fn treesitter_parser_lib_path(&self) -> PathBuf {
    lock!(self.loader).parser_lib_path.clone()
  }

  pub fn set_treesitter_parser_lib_path(&self, parser_lib_path: PathBuf) {
    lock!(self.loader).parser_lib_path = parser_lib_path;
  }
}

impl Debug for SyntaxLoader {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SyntaxLoader")
      .field("parser_lib_path", &lock!(self.loader).parser_lib_path)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub struct SyntaxTreeSitterGrammarMetadata {
  pub name: CompactString,
  pub camelcase: CompactString,
  pub scope: CompactString,
  pub path: PathBuf,
  pub file_types: Vec<CompactString>,
  pub highlights_path: Option<PathBuf>,
  pub highlights_query: Option<String>,
  pub tags_path: Option<PathBuf>,
  pub tags_query: Option<String>,
  pub injections_path: Option<PathBuf>,
  pub injections_query: Option<String>,
  pub injection_regex: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyntaxTreeSitterGrammarRepository {
  pub grammars: Vec<SyntaxTreeSitterGrammarMetadata>,

  /// The `tree-sitter-c`, `tree-sitter-rust` grammar directory path.
  pub grammar_path: PathBuf,

  /// The `src` directory path inside the grammar directory.
  pub src_path: PathBuf,

  /// The `grammar.json` file path inside `src` directory.
  pub grammar_json_path: PathBuf,
}

impl SyntaxLoader {
  pub fn parse_treesitter_grammar_metadata(
    grammar_path: &Path,
  ) -> TheResult<SyntaxTreeSitterGrammarRepository> {
    let err = || {
      TheErr::TreeSitterParserNotFound(
        grammar_path.to_string_lossy().to_compact_string(),
      )
    };

    let tree_sitter_json_path = grammar_path.join("tree-sitter.json");
    let tree_sitter_json_text =
      std::fs::read_to_string(tree_sitter_json_path).map_err(|_e| err())?;
    let tree_sitter_json_data: serde_json::Value =
      serde_json::from_str(&tree_sitter_json_text).map_err(|_e| err())?;

    let tree_sitter_json_grammars = tree_sitter_json_data
      .get("grammars")
      .ok_or(err())?
      .as_array()
      .ok_or(err())?;

    let mut grammars = Vec::with_capacity(tree_sitter_json_grammars.len());
    for ts_grammar in tree_sitter_json_grammars {
      let name = ts_grammar.get("name").ok_or(err())?.as_str().ok_or(err())?;
      let camelcase = ts_grammar
        .get("camelcase")
        .ok_or(err())?
        .as_str()
        .ok_or(err())?;
      let scope = ts_grammar
        .get("scope")
        .ok_or(err())?
        .as_str()
        .ok_or(err())?;
      let path = ts_grammar.get("path").ok_or(err())?.as_str().ok_or(err())?;
      let path = grammar_path.join(path).canonicalize().map_err(|_e| err())?;
      let file_types = ts_grammar
        .get("file-types")
        .ok_or(err())?
        .as_array()
        .ok_or(err())?
        .iter()
        .map(|ft| ft.as_str().ok_or(err()));
      let file_types = process_results(file_types, |ft| ft.collect_vec())?
        .iter()
        .map(|ft| ft.to_compact_string())
        .collect_vec();
      let highlights_path = ts_grammar
        .get("highlights")
        .map(|hl| hl.as_str().ok_or(err()))
        .transpose()?
        .map(|hl| grammar_path.join(hl).normalize().map_err(|_e| err()))
        .transpose()?;
      let tags_path = ts_grammar
        .get("tags")
        .map(|tg| tg.as_str().ok_or(err()))
        .transpose()?
        .map(|tg| grammar_path.join(tg).normalize().map_err(|_e| err()))
        .transpose()?;
      let injections_path = ts_grammar
        .get("injections")
        .map(|inj| inj.as_str().ok_or(err()))
        .transpose()?
        .map(|inj| grammar_path.join(inj).normalize().map_err(|_e| err()))
        .transpose()?;
      let injection_regex = ts_grammar
        .get("injection-regex")
        .map(|tg| tg.as_str().ok_or(err()))
        .transpose()?
        .map(|inj| inj.to_string());
      let metadata = SyntaxTreeSitterGrammarMetadata {
        name: name.to_compact_string(),
        camelcase: camelcase.to_compact_string(),
        scope: scope.to_compact_string(),
        path,
        file_types,
        highlights_path,
        highlights_query: None,
        tags_path,
        tags_query: None,
        injections_path,
        injections_query: None,
        injection_regex,
      };
      grammars.push(metadata);
    }

    let src_path = grammar_path.join("src");
    let grammar_json_path = src_path.join("grammar.json");

    let repository = SyntaxTreeSitterGrammarRepository {
      grammars,
      grammar_path: grammar_path.to_path_buf(),
      src_path,
      grammar_json_path,
    };
    Ok(repository)
  }

  /// Load the tree-sitter parser/grammar (`Language`) FFI dynamic library.
  pub fn load_grammar(
    &self,
    req: &SyntaxLoadGrammarRequest,
  ) -> TheResult<(
    /* metainfo */ SyntaxTreeSitterGrammarRepository,
    /* grammar */ Language,
  )> {
    let metainfo =
      Self::parse_treesitter_grammar_metadata(req.grammar_path.as_path())?;
    let compile_cfg =
      CompileConfig::new(metainfo.src_path.as_path(), None, None);
    match lock!(self.loader).load_language_at_path(compile_cfg) {
      Ok(grammar) => Ok((metainfo, grammar)),
      Err(e) => Err(TheErr::LoadTreeSitterParserFailed(
        req.grammar_path.to_string_lossy().to_compact_string(),
        e,
      )),
    }
  }

  pub async fn async_load_grammar(
    &self,
    req: &SyntaxLoadGrammarRequest,
  ) -> TheResult<(
    /* metainfo */ SyntaxTreeSitterGrammarRepository,
    /* grammar */ Language,
  )> {
    self.load_grammar(req)
  }
}

fn save_loaded_grammars(
  syntax_manager: &SyntaxManagerArc,
  repository: &SyntaxTreeSitterGrammarRepository,
  grammar: &Language,
) {
  for metadata in repository.grammars.iter() {
    let highlight_query = match &metadata.highlights_path {
      Some(highlights) => std::fs::read_to_string(highlights).ok(),
      None => None,
    };
    let tags_query = match &metadata.tags_path {
      Some(tags) => std::fs::read_to_string(tags).ok(),
      None => None,
    };
    let injection_query =
      metadata.injection_regex.as_ref().map(|inj| inj.to_string());
    lock!(syntax_manager).insert_grammar(
      &metadata.name,
      &metadata.file_types,
      Some(grammar.clone()),
      highlight_query,
      tags_query,
      injection_query,
    );
  }
}

async fn async_save_loaded_grammars(
  syntax_manager: &SyntaxManagerArc,
  repository: &SyntaxTreeSitterGrammarRepository,
  grammar: &Language,
) {
  for metadata in repository.grammars.iter() {
    let highlight_query = match &metadata.highlights_path {
      Some(highlights) => tokio::fs::read_to_string(highlights).await.ok(),
      None => None,
    };
    let tags_query = match &metadata.tags_path {
      Some(tags) => tokio::fs::read_to_string(tags).await.ok(),
      None => None,
    };
    let injection_query =
      metadata.injection_regex.as_ref().map(|inj| inj.to_string());
    lock!(syntax_manager).insert_grammar(
      &metadata.name,
      &metadata.file_types,
      Some(grammar.clone()),
      highlight_query,
      tags_query,
      injection_query,
    );
  }
}

pub fn load_syntax_grammar(
  syntax_manager: SyntaxManagerArc,
  req: &SyntaxLoadGrammarRequest,
) -> TheResult<SyntaxTreeSitterGrammarRepository> {
  let syntax_loader = lock!(syntax_manager).loader();
  let (metainfo, grammar) = syntax_loader.load_grammar(req)?;
  save_loaded_grammars(&syntax_manager, &metainfo, &grammar);
  Ok(metainfo)
}

pub async fn async_load_syntax_grammar(
  syntax_manager: SyntaxManagerArc,
  req: &SyntaxLoadGrammarRequest,
) -> TheResult<SyntaxTreeSitterGrammarRepository> {
  let syntax_loader = lock!(syntax_manager).loader();
  let (metainfo, grammar) = syntax_loader.async_load_grammar(req).await?;
  async_save_loaded_grammars(&syntax_manager, &metainfo, &grammar).await;
  Ok(metainfo)
}

pub struct SyntaxManager {
  loader: SyntaxLoaderArc,

  // loaded_parsers: FoldMap<CompactString, SyntaxLoadedParser>,
  grammars: FoldMap<CompactString, Language>,
  highlight_queries: FoldMap<CompactString, String>,
  tags_queries: FoldMap<CompactString, String>,
  injection_queries: FoldMap<CompactString, String>,

  // Maps grammar name to file types
  name2ftypes: FoldMap<CompactString, FoldSet<CompactString>>,
  // Maps file type to grammar name
  ftype2name: FoldMap<CompactString, CompactString>,
}

arc_mutex_ptr!(SyntaxManager);

impl Debug for SyntaxManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SyntaxManager")
      .field("loader", &self.loader)
      .field("grammars", &self.grammars.keys())
      .field("highlight_queries", &self.highlight_queries)
      .field("tags_queries", &self.tags_queries)
      .field("injection_queries", &self.injection_queries)
      .field("name2ftypes", &self.name2ftypes)
      .field("ftype2name", &self.ftype2name)
      .finish()
  }
}

#[cfg(test)]
// WARNING: This is for testing only
struct _BuiltinTreeSitterGrammar {
  pub grammar_name: CompactString,
  pub language: Language,
  pub highlight_query: Option<String>,
  pub tags_query: Option<String>,
  pub injection_query: Option<String>,
  pub file_types: Vec<CompactString>,
}

#[cfg(test)]
impl _BuiltinTreeSitterGrammar {
  pub fn new(
    grammar_name: &str,
    language: Language,
    highlight_query: Option<String>,
    tags_query: Option<String>,
    injection_query: Option<String>,
    file_types: Vec<CompactString>,
  ) -> Self {
    Self {
      grammar_name: grammar_name.to_compact_string(),
      language,
      highlight_query,
      tags_query,
      injection_query,
      file_types,
    }
  }
}

// Language ID and file extensions {
impl SyntaxManager {
  fn _new() -> Self {
    Self {
      loader: SyntaxLoader::to_arc(SyntaxLoader::new()),
      grammars: FoldMap::new(),
      highlight_queries: FoldMap::new(),
      tags_queries: FoldMap::new(),
      injection_queries: FoldMap::new(),
      name2ftypes: FoldMap::new(),
      ftype2name: FoldMap::new(),
    }
  }

  #[cfg(not(test))]
  pub fn new() -> Self {
    Self::_new()
  }

  #[cfg(test)]
  pub fn new() -> Self {
    let mut it = Self::_new();

    let grammar_bindings = [
      _BuiltinTreeSitterGrammar::new(
        "c",
        tree_sitter_c::LANGUAGE.into(),
        Some(tree_sitter_c::HIGHLIGHT_QUERY.to_string()),
        Some(tree_sitter_c::TAGS_QUERY.to_string()),
        None,
        ["c", "h"]
          .iter()
          .map(|ft| ft.to_compact_string())
          .collect_vec(),
      ),
      _BuiltinTreeSitterGrammar::new(
        "rust",
        tree_sitter_rust::LANGUAGE.into(),
        Some(tree_sitter_rust::HIGHLIGHTS_QUERY.to_string()),
        Some(tree_sitter_rust::TAGS_QUERY.to_string()),
        Some(tree_sitter_rust::INJECTIONS_QUERY.to_string()),
        ["rs"].iter().map(|ft| ft.to_compact_string()).collect_vec(),
      ),
      _BuiltinTreeSitterGrammar::new(
        "markdown",
        tree_sitter_md::LANGUAGE.into(),
        Some(tree_sitter_md::HIGHLIGHT_QUERY_BLOCK.to_string()),
        None,
        Some(tree_sitter_md::INJECTION_QUERY_BLOCK.to_string()),
        ["md", "markdown"]
          .iter()
          .map(|ft| ft.to_compact_string())
          .collect_vec(),
      ),
      _BuiltinTreeSitterGrammar::new(
        "toml",
        tree_sitter_toml_ng::LANGUAGE.into(),
        Some(tree_sitter_toml_ng::HIGHLIGHTS_QUERY.to_string()),
        None,
        None,
        ["toml"]
          .iter()
          .map(|ft| ft.to_compact_string())
          .collect_vec(),
      ),
      _BuiltinTreeSitterGrammar::new(
        "html",
        tree_sitter_html::LANGUAGE.into(),
        Some(tree_sitter_html::HIGHLIGHTS_QUERY.to_string()),
        None,
        Some(tree_sitter_html::INJECTIONS_QUERY.to_string()),
        ["html", "htm"]
          .iter()
          .map(|ft| ft.to_compact_string())
          .collect_vec(),
      ),
    ];

    for grammar_binding in grammar_bindings {
      it.insert_grammar(
        &grammar_binding.grammar_name,
        &grammar_binding.file_types,
        Some(grammar_binding.language),
        grammar_binding.highlight_query,
        grammar_binding.tags_query,
        grammar_binding.injection_query,
      );
    }

    it
  }

  pub fn treesitter_parser_lib_path(&self) -> PathBuf {
    self.loader.treesitter_parser_lib_path()
  }

  /// NOTE: This will reset the tree-sitter loader and all loaded
  /// parsers/grammars.
  pub fn set_treesitter_parser_lib_path(&self, parser_lib_path: PathBuf) {
    self.loader.set_treesitter_parser_lib_path(parser_lib_path);
  }

  pub fn loader(&self) -> SyntaxLoaderArc {
    self.loader.clone()
  }

  // /// Associate a grammar ID with a file extension.
  // ///
  // /// For example, a 'C++' grammar can be associate with below file
  // /// extensions:
  // /// - Feader files: hh, h++, hpp
  // /// - Source files: cc, c++, cpp
  // pub fn insert_file_ext(&mut self, id: CompactString, ext: CompactString) {
  //   self
  //     .name2fext
  //     .entry(id.clone())
  //     .or_default()
  //     .insert(ext.clone());
  //   self.fext2name.entry(ext).or_insert(id);
  // }
  //
  // /// Un-associate a grammar ID with a file extension.
  // pub fn remove_file_ext(&mut self, id: &str, ext: &str) {
  //   self
  //     .name2fext
  //     .entry(id.to_compact_string())
  //     .or_default()
  //     .remove(ext);
  //   self.fext2name.remove(ext);
  // }

  pub fn get_file_types_by_grammar_name(
    &self,
    id: &str,
  ) -> Option<&FoldSet<CompactString>> {
    self.name2ftypes.get(id)
  }

  pub fn get_grammar_name_by_file_types(
    &self,
    ext: &str,
  ) -> Option<&CompactString> {
    self.ftype2name.get(ext)
  }
}
// Language ID and file extensions }

// Language and queries {
impl SyntaxManager {
  pub fn insert_grammar(
    &mut self,
    grammar_name: &str,
    file_types: &[CompactString],
    grammar: Option<Language>,
    highlight_query: Option<String>,
    tags_query: Option<String>,
    injection_query: Option<String>,
  ) {
    if let Some(grammar) = grammar {
      self
        .grammars
        .insert(grammar_name.to_compact_string(), grammar);
    }
    if let Some(hl) = highlight_query {
      self
        .highlight_queries
        .insert(grammar_name.to_compact_string(), hl);
    }
    if let Some(tag) = tags_query {
      self
        .tags_queries
        .insert(grammar_name.to_compact_string(), tag);
    }
    if let Some(injection) = injection_query {
      self
        .injection_queries
        .insert(grammar_name.to_compact_string(), injection);
    }
    self
      .name2ftypes
      .entry(grammar_name.to_compact_string())
      .or_default();
    let exts = self.name2ftypes.get_mut(grammar_name).unwrap();
    for ft in file_types.iter() {
      exts.insert(ft.clone());
      self
        .ftype2name
        .insert(ft.clone(), grammar_name.to_compact_string());
    }
  }

  pub fn get_grammar(&self, id: &str) -> Option<&Language> {
    self.grammars.get(id)
  }

  pub fn get_highlight_query(&self, id: &str) -> Option<&String> {
    self.highlight_queries.get(id)
  }

  pub fn get_grammar_by_ext(&self, ext: &str) -> Option<&Language> {
    self
      .ftype2name
      .get(ext)
      .map(|id| self.get_grammar(id))
      .unwrap_or(None)
  }

  pub fn get_highlight_query_by_ext(&self, ext: &str) -> Option<&String> {
    self
      .ftype2name
      .get(ext)
      .map(|id| self.get_highlight_query(id))
      .unwrap_or(None)
  }

  /// Load/create a new Syntax by file extension.
  pub fn make_syntax_by_ext(
    &self,
    file_extension: &Option<CompactString>,
  ) -> TheResult<Option<Syntax>> {
    if let Some(ext) = file_extension
      && let Some(grammar) = self.get_grammar_by_ext(ext)
    {
      trace!(
        "Load syntax by file ext:{:?} grammar:{:?}",
        file_extension,
        grammar.name()
      );
      let highlight_query = self.get_highlight_query_by_ext(ext);
      match Syntax::new(grammar, highlight_query) {
        Ok(syntax) => Ok(Some(syntax)),
        Err(e) => Err(TheErr::LoadSyntaxFailed(ext.clone(), e)),
      }
    } else {
      Ok(None)
    }
  }
}
// Language and queries }

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

/// NOTE: Make this method public only for testing purpose.
pub fn _parse(
  parser: TreeSitterParserArc,
  old_tree: Option<Tree>,
  pending_edits: Vec<SyntaxEdit>,
) -> (Option<Tree>, isize, Option<Rope>, Option<String>) {
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

  let mut last_rope: Option<Rope> = None;
  let mut last_payload: Option<String> = None;

  if !pending_edits.is_empty() && matches!(pending_edits[0], SyntaxEdit::New(_))
  {
    match &pending_edits[0] {
      SyntaxEdit::New(new) => {
        let payload = new.payload.to_string();
        let new_tree = parser.parse(&payload, tree.as_ref());
        tree = new_tree;
        editing_version = new.version;
        // trace!(
        //   "Parsed new tree:{:?}, editing_version:{:?}",
        //   tree
        //     .clone()
        //     .map(|t| t.root_node().to_string())
        //     .unwrap_or("None".to_string()),
        //   editing_version
        // );
        last_rope = Some(new.payload.clone());
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
    // trace!(
    //   "Parsed update tree:{:?}, editing_version:{:?}",
    //   tree
    //     .clone()
    //     .map(|t| t.root_node().to_string())
    //     .unwrap_or("None".to_string()),
    //   editing_version
    // );
    last_rope = Some(last_update.payload.clone());
    last_payload = Some(payload);
  }

  (tree, editing_version, last_rope, last_payload)
}

fn convert_ts_byte(rope: &Rope, byte_idx: usize) -> usize {
  debug_assert!(rope.try_byte_to_char(byte_idx).is_ok());
  rope.byte_to_char(byte_idx)
}

fn convert_ts_point(rope: &Rope, point: &tree_sitter::Point) -> (usize, usize) {
  let line_idx = point.row;
  debug_assert!(rope.get_line(line_idx).is_some());
  let line = rope.line(line_idx);
  debug_assert!(line.try_byte_to_char(point.column).is_ok());
  let char_idx = line.byte_to_char(point.column);
  (line_idx, char_idx)
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
///
/// NOTE: Make this method public only for testing purpose.
pub fn _query(
  tree: &Option<Tree>,
  text_rope: &Option<Rope>,
  text_payload: &Option<String>,
  highlight_query: &Option<TreeSitterQueryArc>,
) -> Option<SyntaxCaptureArc> {
  let mut query_cursor = QueryCursor::new();
  if let Some(syn_tree) = tree
    && let Some(syn_highlight_query) = highlight_query
    && let Some(text_rope) = text_rope
    && let Some(text_payload) = text_payload
  {
    debug_assert_eq!(&text_rope.to_string(), text_payload);
    query_cursor.set_byte_range(0..usize::MAX);
    let mut matches = query_cursor.matches(
      syn_highlight_query,
      syn_tree.root_node(),
      text_payload.as_bytes(),
    );
    let mut nodes: SyntaxCaptureMap = FoldMap::new();
    while let Some(mat) = matches.next() {
      for cap in mat.captures {
        let index = cap.index;
        let name = syn_highlight_query.capture_names()[index as usize];
        let range = cap.node.range();
        trace!(
          "Captured highlight {}: name:{:?}, range:{:?}",
          index, name, range
        );
        debug_assert!(text_rope.get_line(range.start_point.row).is_some());
        debug_assert!(
          text_rope
            .line(range.start_point.row)
            .try_byte_to_char(range.start_point.column)
            .is_ok()
        );
        let (start_line_idx, start_char_idx) =
          convert_ts_point(text_rope, &range.start_point);
        let key = SyntaxCapturePoint {
          line_idx: start_line_idx,
          char_idx: start_char_idx,
        };
        nodes.entry(key).or_insert(SyntaxCaptureMultipleValues {
          values: vec![],
          max_end_char: None,
          max_end_point: None,
        });
        let absolute_start_char_idx =
          convert_ts_byte(text_rope, range.start_byte);
        let absolute_end_char_idx = convert_ts_byte(text_rope, range.end_byte);
        let (end_line_idx, end_char_idx) =
          convert_ts_point(text_rope, &range.end_point);
        let end_point = SyntaxCapturePoint {
          line_idx: end_line_idx,
          char_idx: end_char_idx,
        };
        let val = nodes.get_mut(&key).unwrap();
        val.values.push(SyntaxCaptureValue {
          index,
          name: name.to_compact_string(),
          range: SyntaxCaptureRange {
            start_char: absolute_start_char_idx,
            end_char: absolute_end_char_idx,
            start_point: SyntaxCapturePoint {
              line_idx: start_line_idx,
              char_idx: start_char_idx,
            },
            end_point,
          },
        });
        val.max_end_char = Some(
          val
            .max_end_char
            .map(|c| std::cmp::max(c, absolute_end_char_idx))
            .unwrap_or(absolute_end_char_idx),
        );
        val.max_end_point = Some(
          val
            .max_end_point
            .map(|p| std::cmp::max(p, end_point))
            .unwrap_or(end_point),
        );
      }
    }
    Some(SyntaxCapture::to_arc(SyntaxCapture::new(nodes)))
  } else {
    None
  }
}

pub async fn parse_and_query(
  ts_parser: TreeSitterParserArc,
  old_ts_tree: Option<Tree>,
  ts_highlight_query: Option<TreeSitterQueryArc>,
  pending_edits: Vec<SyntaxEdit>,
) -> (Option<Tree>, isize, Option<SyntaxCaptureArc>) {
  let (new_ts_tree, editing_version, text_rope, text_payload) =
    _parse(ts_parser, old_ts_tree, pending_edits);
  let highlight_capture =
    _query(&new_ts_tree, &text_rope, &text_payload, &ts_highlight_query);
  (new_ts_tree, editing_version, highlight_capture)
}
