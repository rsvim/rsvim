//! Basic unit of terminal frame.

#![allow(dead_code)]

use compact_str::{CompactString, ToCompactString};
use crossterm::style::{Attributes, Color};

#[derive(Debug, Clone, Eq, PartialEq)]
/// Single character/grapheme rendering unit, it accepts ansi/unicode/emoji/nerd font symbol.
pub struct Cell {
  /// The character/grapheme.
  pub symbol: CompactString,
  /// Foreground color.
  pub fg: Color,
  /// Background color.
  pub bg: Color,
  /// Attributes: underline, bold, italic, etc.
  pub attrs: Attributes,
}

impl Cell {
  /// Get symbol.
  pub fn symbol(&self) -> &CompactString {
    &self.symbol
  }

  /// Set symbol.
  pub fn set_symbol(&mut self, symbol: CompactString) {
    self.symbol = symbol;
  }

  /// Set symbol by char.
  pub fn set_char(&mut self, ch: char) {
    self.symbol = ch.to_compact_string();
  }

  /// Set symbol by str.
  pub fn set_str(&mut self, s: &str) {
    self.symbol = CompactString::new(s);
  }

  /// Get foreground color.
  pub fn fg(&self) -> Color {
    self.fg
  }

  /// Set foreground color.
  pub fn set_fg(&mut self, color: Color) {
    self.fg = color;
  }

  /// Get background color.
  pub fn bg(&self) -> Color {
    self.bg
  }

  /// Set background color.
  pub fn set_bg(&mut self, color: Color) {
    self.bg = color;
  }

  /// Get attributes.
  pub fn attrs(&self) -> Attributes {
    self.attrs
  }

  /// Set attributes.
  pub fn set_attrs(&mut self, attrs: Attributes) {
    self.attrs = attrs;
  }
}

impl Default for Cell {
  /// Make cell with a whitespace and no color, empty attributes.
  fn default() -> Self {
    Cell::none()
  }
}

impl Cell {
  /// Make cell with a symbol, foreground/background color, attributes.
  pub fn new(symbol: CompactString, fg: Color, bg: Color, attrs: Attributes) -> Self {
    Cell {
      symbol,
      fg,
      bg,
      attrs,
    }
  }

  /// Make none cell, it's the default invisible cell.
  pub fn none() -> Self {
    Cell {
      symbol: CompactString::new(" "),
      fg: Color::Reset,
      bg: Color::Reset,
      attrs: Attributes::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default1() {
    let c = Cell::default();
    assert_eq!(c.symbol(), " ");
    assert_eq!(c.fg(), Color::Reset);
    assert_eq!(c.bg(), Color::Reset);
    assert_eq!(c.attrs(), Attributes::default());
  }

  #[test]
  fn new1() {
    let c1 = Cell::new(
      CompactString::const_new(" "),
      Color::Reset,
      Color::Reset,
      Attributes::default(),
    );
    let c2 = Cell::default();
    assert_eq!(c1.symbol(), " ");
    assert_eq!(c1.symbol(), c2.symbol());
    assert_eq!(c1.fg(), Color::Reset);
    assert_eq!(c1.fg(), c2.fg());
    assert_eq!(c1.bg(), Color::Reset);
    assert_eq!(c1.bg(), c2.bg());
    assert_eq!(c1.attrs(), Attributes::default());
    assert_eq!(c1.attrs(), c2.attrs());
  }
}
