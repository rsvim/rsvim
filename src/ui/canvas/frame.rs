//! Frame for terminal rendering.

#![allow(dead_code)]

use std::ops::Range;

use crate::cart::{U16Pos, U16Size};
use crate::ui::canvas::frame::cell::Cell;
use crate::ui::canvas::frame::cursor::Cursor;

pub mod cell;
pub mod cursor;

#[derive(Debug, Clone)]
/// Logical frame for the canvas.
///
/// When UI widget tree drawing on the canvas, it actually draws on the current frame. Then the
/// canvas will diff the changes made by UI tree, and only print the changes to hardware device.
pub struct Frame {
  size: U16Size,
  cells: Vec<Cell>,
  cursor: Cursor,

  /// Indicate which part of the frame is dirty, i.e. it's been drawn by the UI widget tree. When
  /// rendering to the hardware device, only dirty parts will be printed.
  dirty_cells: Vec<Range<usize>>,
  dirty_cursor: bool,
}

// Make range from start position and length of following N elements.
fn range_from_pos(pos: U16Pos, n: usize) -> Range<usize> {
  let start_at = pos.x() as usize * pos.y() as usize;
  let end_at = start_at + n;
  start_at..end_at
}

// Make range from start index and length of following N elements.
fn range_from_idx(index: usize, n: usize) -> Range<usize> {
  let end_at = index + n;
  index..end_at
}

impl Frame {
  /// Make new frame.
  pub fn new(size: U16Size, cursor: Cursor) -> Self {
    let n = size.height() as usize * size.width() as usize;
    Frame {
      size,
      cells: vec![Cell::default(); n],
      cursor,
      dirty_cells: vec![], // When first create, it's not dirty.
      dirty_cursor: false,
    }
  }

  /// Get current frame size.
  pub fn size(&self) -> U16Size {
    self.size
  }

  /// Set current frame size.
  pub fn set_size(&mut self, size: U16Size) -> U16Size {
    let old_size = self.size;
    self.size = size;
    self.cells.resize(
      size.height() as usize * size.width() as usize,
      Cell::default(),
    );
    old_size
  }

  /// Get a cell.
  pub fn cell(&self, pos: U16Pos) -> &Cell {
    &self.cells[pos.x() as usize * pos.y() as usize]
  }

  /// Set a cell.
  pub fn set_cell(&mut self, pos: U16Pos, cell: Cell) -> Cell {
    let index = pos.x() as usize * pos.y() as usize;
    let old_cell = self.cells[index].clone();
    self.cells[index] = cell;
    self.dirty_cells.push(range_from_idx(index, 1));
    old_cell
  }

  /// Get all cells.
  pub fn cells(&self) -> &Vec<Cell> {
    &self.cells
  }

  /// Get a range of continuously cells, start from a position and last for N elements.
  pub fn cells_at(&self, pos: U16Pos, n: usize) -> &[Cell] {
    &self.cells[range_from_pos(pos, n)]
  }

  /// Set (replace) cells at a range.
  ///
  /// NOTE: The behavior is almost same with [`Vec::splice`], except use a start position and
  /// following N elements instead of [`Range`].
  ///
  /// Returns old cells.
  pub fn set_cells_at(&mut self, pos: U16Pos, n: usize, cells: Vec<Cell>) -> Vec<Cell> {
    self.dirty_cells.push(range.clone());
    self.cells.splice(range, cells).collect()
  }

  /// Repeatedly set (replace) the same cell at a range.
  ///
  /// NOTE: The behavior is almost same with [`Vec::splice`], except use a start position and
  /// following N elements instead of [`Range`].
  ///
  /// Returns old cells.
  pub fn repeatedly_set_cell_at(&mut self, range: Range<usize>, cell: Cell) -> Vec<Cell> {
    self.dirty_cells.push(range.clone());
    let cells = vec![cell; range.end - range.start];
    self.cells.splice(range, cells).collect()
  }

  /// Get dirty cells.
  pub fn dirty_cells(&self) -> &Vec<Range<usize>> {
    &self.dirty_cells
  }

  /// Get cursor.
  pub fn cursor(&self) -> &Cursor {
    &self.cursor
  }

  /// Set cursor.
  pub fn set_cursor(&mut self, cursor: Cursor) {
    if self.cursor != cursor {
      self.cursor = cursor;
      self.dirty_cursor = true;
    }
  }

  /// Whether cursor is dirty.
  pub fn dirty_cursor(&self) -> bool {
    self.dirty_cursor
  }

  /// Reset/clean all dirty components.
  ///
  /// NOTE: This method should be called after each frame been flushed to terminal device.
  pub fn reset_dirty(&mut self) {
    self.dirty_cells = vec![];
    self.dirty_cursor = false;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new1() {
    let sz = U16Size::new(2, 1);
    let f = Frame::new(sz, Cursor::default());
    assert_eq!(f.size.width, 2);
    assert_eq!(f.size.height, 1);
    assert_eq!(
      f.cells.len(),
      f.size.height as usize * f.size.width as usize
    );
    for c in f.cells.iter() {
      assert_eq!(c.symbol(), Cell::default().symbol());
    }
  }

  #[test]
  fn set_cells1() {
    let sz = U16Size::new(10, 10);
    let _f = Frame::new(sz, Cursor::default());
  }
}
