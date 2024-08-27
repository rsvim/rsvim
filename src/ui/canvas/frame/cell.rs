//! Basic unit of terminal frame.

#![allow(dead_code)]

use compact_str::{CompactString, ToCompactString};
use crossterm::style::{Attributes, Color};

#[derive(Debug, Clone, Eq, PartialEq)]
/// Single character/grapheme rendering unit, it accepts ansi/unicode/emoji/nerd font symbol.
pub struct Cell {
  // The character/grapheme.
  symbol: CompactString,
  // Foreground color.
  fg: Color,
  // Background color.
  bg: Color,
  // Attributes: underline, bold, italic, etc.
  attrs: Attributes,
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
  /// Make cell with empty string.
  fn default() -> Self {
    Cell::empty()
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

  /// Make a space cell.
  pub fn space() -> Self {
    Cell {
      symbol: " ".to_compact_string(),
      fg: Color::Reset,
      bg: Color::Reset,
      attrs: Attributes::default(),
    }
  }

  /// Make an empty cell.
  pub fn empty() -> Self {
    Cell {
      symbol: CompactString::const_new(""),
      fg: Color::Reset,
      bg: Color::Reset,
      attrs: Attributes::default(),
    }
  }
}

impl From<char> for Cell {
  fn from(value: char) -> Self {
    Cell::new(
      value.to_compact_string(),
      Color::Reset,
      Color::Reset,
      Attributes::default(),
    )
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
      CompactString::new(" "),
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

  #[test]
  fn from1() {
    let expects = ['a', 'b', 'c', 'd', 'e', 'F', 'G', 'H', 'I'];
    for (i, input) in expects.iter().enumerate() {
      let c: Cell = (*input).into();
      let s = c.symbol().as_str();
      let cs: Vec<char> = s.chars().collect();
      let expect = expects[i];
      assert!(s.len() == 1);
      assert!(cs.len() == 1);
      assert!(cs[0] == expect);
    }
  }
}
