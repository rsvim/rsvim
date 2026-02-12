#![allow(dead_code, unused_variables)]
//! Highlight.

use crate::prelude::*;
use crate::structural_id_impl;
use compact_str::CompactString;
use crossterm::style::Attributes;
use crossterm::style::Color;

structural_id_impl!(str, StyleId);

#[derive(Debug, Clone, Eq, PartialEq)]
/// Highlight style, including colors and attributes.
pub struct Style {
  /// Style ID
  pub id: StyleId,

  /// Foreground color.
  pub fg: Color,

  /// Background color.
  pub bg: Color,

  /// Attributes: underline, bold, italic, etc.
  pub attr: Attributes,
}

structural_id_impl!(str, HighlightId);

#[derive(Debug, Clone)]
pub struct Highlight {
  // Highlight ID
  id: HighlightId,

  // Maps style ID => style
  styles: FoldMap<StyleId, Style>,
}

impl Highlight {
  pub fn new(id: HighlightId) -> Self {
    Self {
      id,
      styles: FoldMap::new(),
    }
  }
}

#[derive(Debug)]
pub struct HighlightManager {
  // Maps highlight ID => highlight
  highlights: FoldMap<HighlightId, Highlight>,
}

impl Default for HighlightManager {
  fn default() -> Self {
    Self::new()
  }
}

pub type HighlightManagerKeys<'a> =
  std::collections::hash_map::Keys<'a, HighlightId, Highlight>;
pub type HighlightManagerValues<'a> =
  std::collections::hash_map::Values<'a, HighlightId, Highlight>;
pub type HighlightManagerIter<'a> =
  std::collections::hash_map::Iter<'a, HighlightId, Highlight>;

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

  pub fn get(&self, id: &HighlightId) -> Option<&Highlight> {
    self.highlights.get(id)
  }

  pub fn contains_key(&self, id: &HighlightId) -> bool {
    self.highlights.contains_key(id)
  }

  pub fn insert(
    &mut self,
    key: HighlightId,
    value: Highlight,
  ) -> Option<Highlight> {
    self.highlights.insert(key, value)
  }

  pub fn remove(&mut self, id: &HighlightId) -> Option<Highlight> {
    self.highlights.remove(id)
  }
}
