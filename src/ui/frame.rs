//! Terminal rendering frame.

use crate::geo::{U16Pos, U16Rect, U16Size, UPos};
use compact_str::CompactString;
use crossterm::cursor::SetCursorStyle;
use crossterm::style::{Attributes, Color};
use std::vec::Splice;
use std::{cmp, fmt, hash};

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

#[derive(Copy, Clone)]
/// Terminal cursor.
/// Note: This is the real terminal cursor of the device, not a virtual one in multiple cursors.
pub struct Cursor {
  pub pos: U16Pos,
  pub blinking: bool,
  pub hidden: bool,
  pub saved_pos: Option<UPos>,
  pub style: SetCursorStyle,
  pub dirty: bool,
}

struct CursorStyleFormatter {
  style: SetCursorStyle,
}

impl From<SetCursorStyle> for CursorStyleFormatter {
  fn from(style: SetCursorStyle) -> Self {
    CursorStyleFormatter { style }
  }
}

impl fmt::Debug for CursorStyleFormatter {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    write!(f, "{}", self.style)
  }
}

impl Cursor {
  pub fn new(pos: U16Pos, blinking: bool, hidden: bool, style: SetCursorStyle) -> Self {
    Cursor {
      pos,
      blinking,
      hidden,
      saved_pos: None,
      style,
      dirty: true,
    }
  }
}

impl Default for Cursor {
  fn default() -> Self {
    Cursor {
      pos: U16Pos::new(0, 0),
      blinking: false,
      hidden: false,
      saved_pos: None,
      style: SetCursorStyle::DefaultUserShape,
      dirty: true,
    }
  }
}

impl fmt::Debug for Cursor {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    let style_formatter = CursorStyleFormatter::from(self.style);
    f.debug_struct("Cursor")
      .field("pos", &self.pos)
      .field("blinking", &self.blinking)
      .field("hidden", &self.hidden)
      .field("saved_pos", &self.saved_pos)
      .field("style", &style_formatter)
      .finish()
  }
}

impl cmp::PartialEq for Cursor {
  /// Whether equals to other.
  fn eq(&self, other: &Self) -> bool {
    self.pos == other.pos
  }
}

impl cmp::Eq for Cursor {}

impl cmp::PartialOrd for Cursor {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl cmp::Ord for Cursor {
  fn cmp(&self, other: &Self) -> cmp::Ordering {
    self.pos.cmp(&other.pos)
  }
}

impl hash::Hash for Cursor {
  fn hash<H: hash::Hasher>(&self, state: &mut H) {
    self.pos.hash(state);
  }
}

#[derive(Debug, Clone)]
/// Rendering buffer & cursor for the whole terminal.
/// All UI components will dump their text contents to a frame first, then flush to terminal.
pub struct Frame {
  pub size: U16Size,
  pub cells: Vec<Cell>,
  pub cursor: Cursor,
}

impl Frame {
  /// Make new frame.
  pub fn new(size: U16Size, cursor: Cursor) -> Self {
    Frame {
      size,
      cells: vec![Cell::default(); size.height as usize * size.width as usize],
      cursor,
    }
  }

  /// Get a cell on specific position.
  pub fn get_cell(&self, pos: UPos) -> &Cell {
    &self.cells[pos.x * pos.y]
  }

  /// Get a mutable cell on specific position.
  pub fn mut_get_cell(&mut self, pos: UPos) -> &mut Cell {
    &mut self.cells[pos.x * pos.y]
  }

  /// Set a cell on specific position.
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

  /// Set continuously cells, start from position.
  /// Returns n old cells.
  pub fn set_cells(
    &mut self,
    pos: UPos,
    cells: Vec<Cell>,
  ) -> Splice<'_, <Vec<Cell> as IntoIterator>::IntoIter> {
    let start_at = pos.x * pos.y;
    let end_at = start_at + cells.len();
    self.cells.splice(start_at..end_at, cells)
  }

  pub fn get_cursor(&self) -> &Cursor {
    &self.cursor
  }

  pub fn set_cursor(&mut self, cursor: Cursor) {
    self.cursor = cursor;
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
    let b = Frame::new(sz, Cursor::default());
    assert_eq!(b.size.height, 1);
    assert_eq!(b.size.width, 2);
    assert_eq!(b.size.area(), 2);
    assert_eq!(b.cells.len(), b.size.area());
    for c in b.cells.iter() {
      assert_eq!(c.symbol(), Cell::default().symbol());
    }
  }
}
