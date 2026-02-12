//! Highlight.

use crate::prelude::*;
use crate::structural_id_impl;
use crossterm::style::Attributes;
use crossterm::style::Color;

#[derive(Debug, Clone, Eq, PartialEq)]
/// Highlight style, including colors and attributes.
pub struct Style {
  /// Foreground color.
  pub fg: Color,

  /// Background color.
  pub bg: Color,

  /// Attributes: underline, bold, italic, etc.
  pub attr: Attributes,
}

structural_id_impl!(str, HighlightId);

pub struct Highlight {}

arc_mutex_ptr!(Highlight);

pub struct HighlightManager {}

arc_mutex_ptr!(HighlightManager);
