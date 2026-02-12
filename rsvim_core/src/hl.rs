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

#[derive(Debug)]
pub struct HighlightManager {
  // Maps highlight ID => highlight
  highlights: FoldMap<HighlightId, Highlight>,
}
