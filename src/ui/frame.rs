//! Frame for terminal rendering.

#![allow(dead_code)]

pub mod cell;
pub mod cursor;

use crate::cart::{U16Size, UPos};
use std::vec::Splice;

// Re-export
pub use crate::ui::frame::cell::Cell;
pub use crate::ui::frame::cursor::{Cursor, CursorStyle};

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
    &self.cells[pos.x() * pos.y()]
  }

  /// Get a mutable cell on specific position.
  pub fn get_cell_mut(&mut self, pos: UPos) -> &mut Cell {
    &mut self.cells[pos.x() * pos.y()]
  }

  /// Set a cell on specific position.
  pub fn set_cell(&mut self, pos: UPos, cell: Cell) -> &mut Self {
    self.cells[pos.x() * pos.y()] = cell;
    self
  }

  /// Get n continuously cells, start from position.
  pub fn get_cells(&self, pos: UPos, n: usize) -> &[Cell] {
    let start_at = pos.x() * pos.y();
    let end_at = start_at + n;
    &self.cells[start_at..end_at]
  }

  /// Get n continuously mutable cells, start from position.
  pub fn get_cells_mut(&mut self, pos: UPos, n: usize) -> &mut [Cell] {
    let start_at = pos.x() * pos.y();
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
    let start_at = pos.x() * pos.y();
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
  use crossterm::style::{Attributes, Color};

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
    let sz = U16Size::new(1, 2);
    let b = Frame::new(sz, Cursor::default());
    assert_eq!(b.size.height, 1);
    assert_eq!(b.size.width, 2);
    assert_eq!(
      b.cells.len(),
      b.size.height as usize * b.size.width as usize
    );
    for c in b.cells.iter() {
      assert_eq!(c.symbol(), Cell::default().symbol());
    }
  }
}
