//! Terminal rendering frame.

use crate::geo::pos::UPos;
use crate::geo::size::Size;
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

#[derive(Debug, Clone)]
/// Rendering buffer, all UI components will first write symbols/grapheme/characters to a buffer,
/// then flushed to terminal. Terminal will save the previous buffer after flushed, and use it to
/// diff with next buffer, to find out difference and only flush those changed/dirty parts to
/// backend device.
pub struct Frame {
  pub size: Size,
  pub cells: Vec<Cell>,
}

impl Frame {
  /// Make new buffer with [size](crate::geo::size::Size).
  pub fn new(size: Size) -> Self {
    Frame {
      size,
      cells: vec![Cell::default(); size.area()],
    }
  }

  /// Get the cell.
  pub fn get_cell(&self, pos: UPos) -> &Cell {
    &self.cells[pos.x * pos.y]
  }

  /// Get the mutable cell.
  pub fn mut_get_cell(&mut self, pos: UPos) -> &mut Cell {
    &mut self.cells[pos.x * pos.y]
  }

  /// Set the cell.
  pub fn set_cell(&mut self, pos: UPos, cell: Cell) -> &mut Self {
    self.cells[pos.x * pos.y] = cell;
    self
  }

  /// Get n continuously cells, start from position.
  pub fn get_cells(&self, pos: UPos, n: usize) -> &[Cell] {
    let start_at = pos.x * pos.y;
    let end_at = start_at + n;
    &self.cells[start_at..end_at]
  }

  /// Get n continuously mutable cells, start from position.
  pub fn mut_get_cells(&mut self, pos: UPos, n: usize) -> &mut [Cell] {
    let start_at = pos.x * pos.y;
    let end_at = start_at + n;
    &mut self.cells[start_at..end_at]
  }

  /// Set n continuously cells, start from position.
  /// Returns n old cells.
  pub fn set_cells(&mut self, pos: UPos, cells: Vec<Cell>) -> &mut Self {
    let start_at = pos.x * pos.y;
    let end_at = start_at + cells.len();
    self.cells.splice(start_at..end_at, cells);
    self
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

  #[test]
  fn should_equal_on_buffer_new() {
    let sz = Size::new(1, 2);
    let b = Frame::new(sz);
    assert_eq!(b.size.height, 1);
    assert_eq!(b.size.width, 2);
    assert_eq!(b.size.area(), 2);
    assert_eq!(b.cells.len(), b.size.area());
    for c in b.cells.iter() {
      assert_eq!(c.symbol(), Cell::default().symbol());
    }
  }
}
