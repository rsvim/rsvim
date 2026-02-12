//! ColorScheme.

use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::style::Attributes;
use crossterm::style::Color;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct ColorScheme {
  // Highlight name 
  name: CompactString,

  // Maps color name to RGB value.
  // For example: white => #ffffff, black => #000000
  palette: FoldMap<CompactString, CompactString>,

  // Maps ID => syntax colors
  syntax: FoldMap<CompactString, Style>,

  // Maps ID => UI colors
  ui: FoldMap<CompactString, Style>,
}

impl ColorScheme {
  pub fn new(id: CompactString) -> Self {
    Self {
      name: id,
      palette: FoldMap::new(),
      syntax: FoldMap::new(),
      ui: FoldMap::new(),
    }
  }

  pub fn from_toml(id: CompactString, )

  pub fn id(&self) -> &CompactString {
    &self.name
  }

  pub fn palette(&self) -> &FoldMap<CompactString, CompactString> {
    &self.palette
  }

  pub fn palette_mut(&mut self) -> &mut FoldMap<CompactString, CompactString> {
    &mut self.palette
  }

  pub fn syntax(&self) -> &FoldMap<CompactString, Style> {
    if cfg!(debug_assertions) {
      for k in self.syntax.keys() {
        debug_assert!(k.starts_with(SYNTAX_PREFIX));
      }
    }
    &self.syntax
  }

  pub fn syntax_mut(&mut self) -> &mut FoldMap<CompactString, Style> {
    &mut self.syntax
  }

  pub fn ui(&self) -> &FoldMap<CompactString, Style> {
    if cfg!(debug_assertions) {
      for k in self.ui.keys() {
        debug_assert!(k.starts_with(UI_PREFIX));
      }
    }
    &self.ui
  }

  pub fn ui_mut(&self) -> &FoldMap<CompactString, Style> {
    &self.ui
  }
}

#[derive(Debug)]
pub struct ColorSchemeManager {
  // Maps highlight ID => highlight
  highlights: FoldMap<CompactString, ColorScheme>,
}

impl Default for ColorSchemeManager {
  fn default() -> Self {
    Self::new()
  }
}

pub type HighlightManagerKeys<'a> =
  std::collections::hash_map::Keys<'a, CompactString, ColorScheme>;
pub type HighlightManagerValues<'a> =
  std::collections::hash_map::Values<'a, CompactString, ColorScheme>;
pub type HighlightManagerIter<'a> =
  std::collections::hash_map::Iter<'a, CompactString, ColorScheme>;

impl ColorSchemeManager {
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

  pub fn get(&self, id: &CompactString) -> Option<&ColorScheme> {
    self.highlights.get(id)
  }

  pub fn contains_key(&self, id: &CompactString) -> bool {
    self.highlights.contains_key(id)
  }

  pub fn insert(
    &mut self,
    key: CompactString,
    value: ColorScheme,
  ) -> Option<ColorScheme> {
    self.highlights.insert(key, value)
  }

  pub fn remove(&mut self, id: &CompactString) -> Option<ColorScheme> {
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

