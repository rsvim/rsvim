use compact_str::CompactString;
use crossterm::style::{Attributes, Color};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Cell {
  symbol: CompactString,
  fg: Color,
  bg: Color,
  attrs: Attributes,
}

impl Cell {
  pub fn symbol(&self) -> &str {
    self.symbol.as_str()
  }

  pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
    self.symbol = CompactString::new(symbol);
    self
  }

  pub fn set_char_symbol(&mut self, ch: char) -> &mut Self {
    let mut buf = [0; 4];
    self.symbol = CompactString::new(ch.encode_utf8(&mut buf));
    self
  }

  pub fn fg(&self) -> Color {
    self.fg
  }

  pub fn set_fg(&mut self, color: Color) -> &mut Self {
    self.fg = color;
    self
  }

  pub fn bg(&self) -> Color {
    self.bg
  }

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
