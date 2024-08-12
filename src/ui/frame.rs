//! Frame for terminal rendering.

#![allow(dead_code)]

use std::ops::Range;

use crate::cart::{U16Size, UPos};
use std::vec::Splice;

// Re-export
pub use crate::ui::frame::cell::Cell;
pub use crate::ui::frame::cursor::{Cursor, CursorStyle, CursorStyleFormatter};

pub mod cell;
pub mod cursor;

pub type FrameCellsRange = Range<usize>;

#[derive(Debug, Clone)]
/// Rendering buffer & cursor for the whole terminal.
/// All UI components will dump their text contents to a frame first, then flush to terminal.
pub struct Frame {
  pub size: U16Size,
  pub cells: Vec<Cell>,
  pub cursor: Cursor,

  /// Indicate which part of the frame is dirty, i.e. been updated by widget tree changes.
  /// When rendering contents to the terminal device, only the dirty ranges will be printed.
  pub dirty_cells: Vec<FrameCellsRange>,
  pub dirty_cursor: bool,
}

impl Frame {
  /// Make new frame.
  pub fn new(size: U16Size, cursor: Cursor) -> Self {
    Frame {
      size,
      cells: vec![Cell::default(); size.height as usize * size.width as usize],
      cursor,
      dirty_cells: vec![],
      dirty_cursor: false,
    }
  }

  /// Get the size.
  pub fn size(&self) -> U16Size {
    self.size
  }

  /// Set the size, i.e. change the frame size.
  pub fn set_size(&mut self, size: U16Size) -> U16Size {
    let old_size = self.size;
    self.size = size;
    old_size
  }

  /// Get a cell on specific position.
  pub fn get_cell(&self, pos: UPos) -> &Cell {
    &self.cells[pos.x() * pos.y()]
  }

  /// Set a cell on specific position.
  pub fn set_cell(&mut self, pos: UPos, cell: Cell) -> Cell {
    let index = pos.x() * pos.y();
    let old = self.cells[index].clone();
    self.cells[index] = cell;
    self.dirty_cells.push(FrameCellsRange {
      start: index,
      end: index + 1,
    });
    old
  }

  /// Get n continuously cells, start from position.
  pub fn get_cells(&self, pos: UPos, n: usize) -> &[Cell] {
    let start_at = pos.x() * pos.y();
    let end_at = start_at + n;
    &self.cells[start_at..end_at]
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
    self.dirty_cells.push(FrameCellsRange {
      start: start_at,
      end: end_at,
    });
    self.cells.splice(start_at..end_at, cells)
  }

  /// Get dirty cells.
  pub fn dirty_cells(&self) -> &Vec<FrameCellsRange> {
    &self.dirty_cells
  }

  /// Get cursor.
  pub fn get_cursor(&self) -> &Cursor {
    &self.cursor
  }

  /// Set cursor.
  pub fn set_cursor(&mut self, cursor: Cursor) {
    self.cursor = cursor;
    self.dirty_cursor = true;
  }

  /// Whether cursor is dirty.
  pub fn dirty_cursor(&self) -> bool {
    self.dirty_cursor
  }

  /// Reset/clean all dirty components.
  ///
  /// Note: This method should be called after each frame been flushed to terminal device.
  pub fn reset_dirty(&mut self) {
    self.dirty_cells = vec![];
    self.dirty_cursor = false;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crossterm::style::{Attributes, Color};

  #[test]
  fn new_cell() {
    let c = Cell::default();
    assert_eq!(c.symbol(), " ");
    assert_eq!(c.fg(), Color::Reset);
    assert_eq!(c.bg(), Color::Reset);
    assert_eq!(c.attrs(), Attributes::default());
  }

  #[test]
  fn new_frame() {
    let sz = U16Size::new(2, 1);
    let b = Frame::new(sz, Cursor::default());
    assert_eq!(b.size.width, 2);
    assert_eq!(b.size.height, 1);
    assert_eq!(
      b.cells.len(),
      b.size.height as usize * b.size.width as usize
    );
    for c in b.cells.iter() {
      assert_eq!(c.symbol(), Cell::default().symbol());
    }
  }
}
