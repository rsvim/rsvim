#![allow(dead_code, unused_variables)]
//! Highlight.

use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::style::Attributes;
use crossterm::style::Color;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
/// Highlight style, including colors and attributes.
pub struct Style {
  /// Style ID
  pub id: CompactString,

  /// Foreground color.
  pub fg: Color,

  /// Background color.
  pub bg: Color,

  /// Attributes: underline, bold, italic, etc.
  pub attr: Attributes,
}

pub static SYNTAX_NAMES: Lazy<FoldSet<CompactString>> = Lazy::new(|| {
  vec![
    "attribute",
    "boolean",
    "carriage-return",
    "comment",
    "comment.documentation",
    "constant",
    "constant.builtin",
    "constructor",
    "constructor.builtin",
    "embedded",
    "error",
    "escape",
    "function",
    "function.builtin",
    "keyword",
    "markup",
    "markup.bold",
    "markup.heading",
    "markup.italic",
    "markup.link",
    "markup.link.url",
    "markup.list",
    "markup.list.checked",
    "markup.list.numbered",
    "markup.list.unchecked",
    "markup.list.unnumbered",
    "markup.quote",
    "markup.raw",
    "markup.raw.block",
    "markup.raw.inline",
    "markup.strikethrough",
    "module",
    "number",
    "operator",
    "property",
    "property.builtin",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.escape",
    "string.regexp",
    "string.special",
    "string.special.symbol",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
  ]
  .iter()
  .map(|i| i.to_compact_string())
  .collect::<FoldSet<CompactString>>()
});

#[derive(Debug, Clone)]
pub struct Highlight {
  // Highlight ID
  id: CompactString,

  // Maps ID => syntax colors
  syntax: FoldMap<CompactString, Style>,

  // Maps ID => UI colors
  ui: FoldMap<CompactString, Style>,
}

impl Highlight {
  pub fn new(id: CompactString) -> Self {
    Self {
      id,
      syntax: FoldMap::new(),
      ui: FoldMap::new(),
    }
  }

  pub fn id(&self) -> &CompactString {
    &self.id
  }

  pub fn syntax(&self) -> &FoldMap<CompactString, Style> {
    &self.syntax
  }

  pub fn syntax_mut(&mut self) -> &mut FoldMap<CompactString, Style> {
    &mut self.syntax
  }

  pub fn ui(&self) -> &FoldMap<CompactString, Style> {
    &self.ui
  }

  pub fn ui_mut(&self) -> &FoldMap<CompactString, Style> {
    &self.ui
  }
}

#[derive(Debug)]
pub struct HighlightManager {
  // Maps highlight ID => highlight
  highlights: FoldMap<CompactString, Highlight>,
}

impl Default for HighlightManager {
  fn default() -> Self {
    Self::new()
  }
}

pub type HighlightManagerKeys<'a> =
  std::collections::hash_map::Keys<'a, CompactString, Highlight>;
pub type HighlightManagerValues<'a> =
  std::collections::hash_map::Values<'a, CompactString, Highlight>;
pub type HighlightManagerIter<'a> =
  std::collections::hash_map::Iter<'a, CompactString, Highlight>;

impl HighlightManager {
  pub fn new() -> Self {
    Self {
      highlights: FoldMap::new(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.highlights.is_empty()
  }

  pub fn len(&self) -> usize {
    self.highlights.len()
  }

  pub fn get(&self, id: &CompactString) -> Option<&Highlight> {
    self.highlights.get(id)
  }

  pub fn contains_key(&self, id: &CompactString) -> bool {
    self.highlights.contains_key(id)
  }

  pub fn insert(
    &mut self,
    key: CompactString,
    value: Highlight,
  ) -> Option<Highlight> {
    self.highlights.insert(key, value)
  }

  pub fn remove(&mut self, id: &CompactString) -> Option<Highlight> {
    self.highlights.remove(id)
  }

  pub fn keys(&self) -> HighlightManagerKeys<'_> {
    self.highlights.keys()
  }

  pub fn values(&self) -> HighlightManagerValues<'_> {
    self.highlights.values()
  }

  pub fn iter(&self) -> HighlightManagerIter<'_> {
    self.highlights.iter()
  }
}
