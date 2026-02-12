#![allow(dead_code, unused_variables)]
//! Highlight.

use crate::prelude::*;
use crate::structural_id_impl;
use compact_str::CompactString;
use crossterm::style::Attributes;
use crossterm::style::Color;

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

#[derive(Debug, Clone)]
pub struct Highlight {
  // Highlight ID
  id: CompactString,

  // Maps style ID => style
  styles: FoldMap<CompactString, Style>,
}

pub type HighlightKeys<'a> =
  std::collections::hash_map::Keys<'a, CompactString, Style>;
pub type HighlightValues<'a> =
  std::collections::hash_map::Values<'a, CompactString, Style>;
pub type HighlightIter<'a> =
  std::collections::hash_map::Iter<'a, CompactString, Style>;

impl Highlight {
  pub fn new(id: CompactString) -> Self {
    Self {
      id,
      styles: FoldMap::new(),
    }
  }

  pub fn id(&self) -> &CompactString {
    &self.id
  }

  pub fn is_empty(&self) -> bool {
    self.styles.is_empty()
  }

  pub fn len(&self) -> usize {
    self.styles.len()
  }

  pub fn get(&self, id: &str) -> Option<&Style> {
    self.styles.get(id)
  }

  pub fn contains_key(&self, id: &str) -> bool {
    self.styles.contains_key(id)
  }

  pub fn insert(&mut self, key: CompactString, value: Style) -> Option<Style> {
    self.styles.insert(key, value)
  }

  pub fn remove(&mut self, id: &str) -> Option<Style> {
    self.styles.remove(id)
  }

  pub fn keys(&self) -> HighlightKeys<'_> {
    self.styles.keys()
  }

  pub fn values(&self) -> HighlightValues<'_> {
    self.styles.values()
  }

  pub fn iter(&self) -> HighlightIter<'_> {
    self.styles.iter()
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
