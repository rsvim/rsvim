//! Basic unit of terminal frame.

#![allow(dead_code)]

use compact_str::CompactString;
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
  /// Indicates whether this cell is been modified, other UI components will modify a cell and make
  /// it dirty, and it comes back to clean after been flushed to terminal.
  pub dirty: bool,
}

impl Cell {
  /// Get symbol.
  pub fn symbol(&self) -> &str {
    self.symbol.as_str()
  }

  /// Set symbol.
  pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
    self.symbol = CompactString::new(symbol);
    self.dirty = true;
    self
  }

  /// Set symbol by char.
  pub fn set_char(&mut self, ch: char) -> &mut Self {
    let mut buf = [0; 4];
    self.symbol = CompactString::new(ch.encode_utf8(&mut buf));
    self.dirty = true;
    self
  }

  /// Get foreground color.
  pub fn fg(&self) -> Color {
    self.fg
  }

  /// Set foreground color.
  pub fn set_fg(&mut self, color: Color) -> &mut Self {
    self.fg = color;
    self.dirty = true;
    self
  }

  /// Get background color.
  pub fn bg(&self) -> Color {
    self.bg
  }

  /// Set background color.
  pub fn set_bg(&mut self, color: Color) -> &mut Self {
    self.bg = color;
    self.dirty = true;
    self
  }

  /// Get attributes.
  pub fn attrs(&self) -> Attributes {
    self.attrs
  }

  /// Set attributes.
  pub fn set_attrs(&mut self, attrs: Attributes) -> &mut Self {
    self.attrs = attrs;
    self.dirty = true;
    self
  }

  /// Indicate whether this cell is dirty.
  pub fn dirty(&self) -> bool {
    self.dirty
  }
}

impl Default for Cell {
  /// Make cell with a whitespace and no color, empty attributes.
  fn default() -> Self {
    Cell {
      symbol: CompactString::new_inline(" "),
      fg: Color::Reset,
      bg: Color::Reset,
      attrs: Attributes::default(),
      dirty: true,
    }
  }
}

impl Cell {
  fn new(symbol: CompactString, fg: Color, bg: Color, attrs: Attributes) -> Self {
    Cell {
      symbol,
      fg,
      bg,
      attrs,
      dirty: true,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_cell_default() {
    let c = Cell::default();
    assert_eq!(c.symbol(), " ");
    assert_eq!(c.fg(), Color::Reset);
    assert_eq!(c.bg(), Color::Reset);
    assert_eq!(c.attrs(), Attributes::default());
  }
}
