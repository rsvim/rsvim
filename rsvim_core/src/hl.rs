//! Highlight.

pub mod style;
pub mod syn;

use crate::prelude::*;
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
  pub attrs: Attributes,
}

pub struct Highlight {}

arc_mutex_ptr!(Highlight);

pub struct HighlightManager {}

arc_mutex_ptr!(HighlightManager);
