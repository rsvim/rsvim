use compact_str::CompactString;
use crossterm::style::{Attributes, Color};

#[derive(Debug, Clone, Eq, PartialEq)]
/// Single character/grapheme rendering unit, it accepts ansi/unicode/emoji/nerd font symbol.
///
/// * `symbol`: The character/grapheme.
/// * `fg`: Foreground color.
/// * `bg`: Background color.
/// * `attrs`: Attributes: underline, bold, italic, etc.
/// * `dirty`: Whether it's been modified, other UI components will modify a cell and make it
/// dirty, and it revert to clean after been flushed to terminal.
pub struct Cell {
  pub symbol: CompactString,
  pub fg: Color,
  pub bg: Color,
  pub attrs: Attributes,
  pub dirty: bool,
}

impl Cell {
  /// symbol getter
  pub fn symbol(&self) -> &str {
    self.symbol.as_str()
  }

  /// symbol setter
  pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
    self.symbol = CompactString::new(symbol);
    self
  }

  /// symbol setter (by char)
  pub fn set_char(&mut self, ch: char) -> &mut Self {
    let mut buf = [0; 4];
    self.symbol = CompactString::new(ch.encode_utf8(&mut buf));
    self
  }

  /// fg getter
  pub fn fg(&self) -> Color {
    self.fg
  }

  /// fg setter
  pub fn set_fg(&mut self, color: Color) -> &mut Self {
    self.fg = color;
    self
  }

  /// bg getter
  pub fn bg(&self) -> Color {
    self.bg
  }

  /// bg setter
  pub fn set_bg(&mut self, color: Color) -> &mut Self {
    self.bg = color;
    self
  }

  pub fn attrs(&self) -> Attributes {
    self.attrs
  }

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
