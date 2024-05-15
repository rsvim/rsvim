//! Basic unit for single character/grapheme rendering.

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
  /// it dirty, and it revert to clean after been flushed to terminal.
  pub dirty: bool,
}

impl Cell {
  /// Symbol getter.
  pub fn symbol(&self) -> &str {
    self.symbol.as_str()
  }

  /// Symbol setter.
  pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
    self.symbol = CompactString::new(symbol);
    self
  }

  /// Symbol setter (by char).
  pub fn set_char(&mut self, ch: char) -> &mut Self {
    let mut buf = [0; 4];
    self.symbol = CompactString::new(ch.encode_utf8(&mut buf));
    self
  }

  /// Foreground color getter.
  pub fn fg(&self) -> Color {
    self.fg
  }

  /// Foreground color setter.
  pub fn set_fg(&mut self, color: Color) -> &mut Self {
    self.fg = color;
    self
  }

  /// Background color getter.
  pub fn bg(&self) -> Color {
    self.bg
  }

  /// Background color setter.
  pub fn set_bg(&mut self, color: Color) -> &mut Self {
    self.bg = color;
    self
  }

  /// Attributes setter.
  pub fn attrs(&self) -> Attributes {
    self.attrs
  }

  /// Attributes setter.
  pub fn set_attrs(&mut self, attrs: Attributes) -> &mut Self {
    self.attrs = attrs;
    self
  }
}

impl Default for Cell {
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
