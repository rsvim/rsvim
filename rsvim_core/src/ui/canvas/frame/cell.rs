//! Basic unit of canvas frame.

use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::style::Attributes;
use crossterm::style::Color;

#[derive(Debug, Clone, Eq, PartialEq)]
/// Single character/grapheme rendering unit, it accepts ansi/unicode/emoji/nerd font symbol.
pub struct Cell {
  // The character/grapheme.
  symbol: CompactString,

  // Foreground and background colors.
  fg: Color,
  bg: Color,

  // Attributes: underline, bold, italic, etc.
  attr: Attributes,
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
  pub fn fg(&self) -> &Color {
    &self.fg
  }

  /// Set foreground color.
  pub fn set_fg(&mut self, color: Color) {
    self.fg = color;
  }

  /// Get background color.
  pub fn bg(&self) -> &Color {
    &self.bg
  }

  /// Set background color.
  pub fn set_bg(&mut self, color: Color) {
    self.bg = color;
  }

  /// Get attributes.
  pub fn attr(&self) -> &Attributes {
    &self.attr
  }

  /// Set attributes.
  pub fn set_attr(&mut self, value: Attributes) {
    self.attr = value;
  }
}

impl Default for Cell {
  /// Make default cell, same with [`Cell::empty()`].
  fn default() -> Self {
    Cell::empty()
  }
}

impl Cell {
  /// Make cell with a symbol, foreground/background color, attributes.
  pub fn new(
    symbol: CompactString,
    fg: Color,
    bg: Color,
    attr: Attributes,
  ) -> Self {
    Cell {
      symbol,
      fg,
      bg,
      attr,
    }
  }

  /// Make a space cell.
  pub fn space() -> Self {
    Cell {
      symbol: " ".to_compact_string(),
      fg: Color::Reset,
      bg: Color::Reset,
      attr: Attributes::default(),
    }
  }

  /// Make an empty cell.
  pub fn empty() -> Self {
    Cell {
      symbol: CompactString::const_new(""),
      fg: Color::Reset,
      bg: Color::Reset,
      attr: Attributes::default(),
    }
  }

  pub fn with_char(c: char) -> Self {
    Cell {
      symbol: c.to_compact_string(),
      fg: Color::Reset,
      bg: Color::Reset,
      attr: Attributes::default(),
    }
  }

  pub fn with_symbol(s: CompactString) -> Self {
    Cell {
      symbol: s,
      fg: Color::Reset,
      bg: Color::Reset,
      attr: Attributes::default(),
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
