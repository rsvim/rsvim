use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::Weak;
use tree_sitter::Language;
use tree_sitter_loader::CompileConfig;
use tree_sitter_loader::Loader;

pub type SyntaxLoaderArc = Arc<Mutex<Loader>>;
pub type SyntaxLoaderWk = Weak<Mutex<Loader>>;
pub type SyntaxLoaderMutexGuard<'a> = MutexGuard<'a, Loader>;

pub struct SyntaxParserLoader {
  // tree-sitter loader
  loader: Loader,

  // tree-sitter parsers
  parsers: FoldMap<CompactString, Language>,
}

arc_mutex_ptr!(SyntaxParserLoader);

#[derive(Debug, Clone)]
pub struct SyntaxParserLoadOptions {
  pub grammar_path: PathBuf,
}

impl SyntaxParserLoader {
  pub fn new() -> Self {
    Self {
      // loader: Arc::new(Mutex::new(Loader::new().unwrap())),
      loader: Loader::new().unwrap(),
      parsers: FoldMap::new(),
    }
  }

  pub fn get_language_name_from_src_path(
    src_path: &Path,
  ) -> TheResult<CompactString> {
    let grammar_json_path = src_path.join("grammar.json");
    let grammar_json_path = grammar_json_path.as_path();
    let err = || {
      TheErr::TreesitterParserNotFound(
        grammar_json_path.to_string_lossy().to_compact_string(),
      )
    };
    let grammar_json_text =
      std::fs::read_to_string(grammar_json_path).map_err(|_e| err())?;
    let grammar_json_data: serde_json::Value =
      serde_json::from_str(&grammar_json_text).map_err(|_e| err())?;
    match grammar_json_data.get("name") {
      Some(name_value) => match name_value.as_str() {
        Some(name) => Ok(name.to_compact_string()),
        None => Err(err()),
      },
      None => Err(err()),
    }
  }

  /// Load the tree-sitter parser (`Language`) FFI dynamic library.
  pub fn load_treesitter_parser(
    &mut self,
    opts: &SyntaxParserLoadOptions,
  ) -> TheResult<&Language> {
    let src_path = opts.grammar_path.join("src");
    let src_path = src_path.as_path();
    let lang_name = Self::get_language_name_from_src_path(src_path)?;
    if !self.parsers.contains_key(&lang_name) {
      let compile_cfg = CompileConfig::new(src_path, None, None);
      match self.loader.load_language_at_path(compile_cfg) {
        Ok(lang) => {
          self.parsers.insert(lang_name.to_compact_string(), lang);
        }
        Err(e) => {
          let e = TheErr::LoadTreesitterParserFailed(
            lang_name.to_compact_string(),
            e,
          );
          return Err(e);
        }
      }
    }
    Ok(self.parsers.get(&lang_name).unwrap())
  }
}

impl Debug for SyntaxParserLoader {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SyntaxParserLoader")
      .field("loader.parser_lib_path", &self.loader.parser_lib_path)
      .field("parsers", &self.parsers)
      .finish()
  }
}
