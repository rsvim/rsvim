//! Style, including colors and attributes.

use crossterm::style::Attributes;
use crossterm::style::Color;

#[derive(Debug, Clone, Eq, PartialEq)]
/// Highlight style, including colors and attributes.
pub struct Style {
  // Foreground color.
  fg: Color,
  // Background color.
  bg: Color,
  // Attributes: underline, bold, italic, etc.
  attrs: Attributes,
}
