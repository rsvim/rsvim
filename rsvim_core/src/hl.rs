//! Highlight.

use crate::prelude::*;
use crate::structural_id_impl;
use compact_str::CompactString;
use crossterm::style::Attributes;
use crossterm::style::Color;

structural_id_impl!(str, HighlightId);

#[derive(Debug, Clone, Eq, PartialEq)]
/// Highlight style, including colors and attributes.
pub struct Style {
  pub id: HighlightId,

  /// Foreground color.
  pub fg: Color,

  /// Background color.
  pub bg: Color,

  /// Attributes: underline, bold, italic, etc.
  pub attr: Attributes,
}

pub struct Highlight {
  styles: FoldMap<HighlightId, Style>,
}

arc_mutex_ptr!(Highlight);

pub struct HighlightManager {}

arc_mutex_ptr!(HighlightManager);
